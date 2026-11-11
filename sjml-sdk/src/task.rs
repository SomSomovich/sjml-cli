//! Single-threaded guest-future executor for `wasm32-unknown-unknown`.
//!
//! The executor never blocks or spins. It polls a future once per host dispatch;
//! a stored waker calls `sjml_async_v1.wake`, which queues a later poll on the
//! host-selected lane. Physical-client module futures run on `CLIENT_MAIN`, and
//! logical-server futures run on `SERVER_MAIN`. Guest tasks are local to one
//! WASM instance and need not be `Send` because WebAssembly threads are not part
//! of ABI v1.

use crate::{ErrorCode, SjmlError, SjmlResult};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
#[cfg(any(target_arch = "wasm32", test))]
use core::task::{RawWaker, RawWakerVTable, Waker};
#[cfg(any(target_arch = "wasm32", test))]
use std::collections::HashMap;
#[cfg(any(target_arch = "wasm32", test))]
use std::rc::Rc;
use std::time::Duration;

mod compute;
mod service;
pub use compute::{spawn_compute, ComputeFuture, MAX_COMPUTE_BYTES};
pub use service::{service_call, ServiceCallFuture};

/// Yields the current guest task and asks the host to schedule its next poll.
///
/// Split CPU work into bounded chunks and await this between chunks so one guest
/// task does not keep a game-thread poll running for the full workload.
pub async fn yield_now() {
    let mut yielded = false;
    core::future::poll_fn(|context| {
        if yielded {
            Poll::Ready(())
        } else {
            yielded = true;
            context.waker().wake_by_ref();
            Poll::Pending
        }
    })
    .await
}

/// Monotonic, module-local u32 identity for one guest future.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct GuestTaskId(u32);

impl GuestTaskId {
    pub const fn get(self) -> u32 {
        self.0
    }

    #[cfg(any(target_arch = "wasm32", test))]
    fn new(value: u32) -> SjmlResult<Self> {
        if value == 0 {
            return Err(SjmlError::new(
                ErrorCode::InvalidHandle,
                "guest task id zero is reserved",
            ));
        }
        Ok(Self(value))
    }
}

/// A host-timed delay that wakes the current guest task when it elapses.
pub struct Sleep {
    duration: Duration,
    #[cfg(target_arch = "wasm32")]
    timer_id: Option<i32>,
}

/// Creates a nonblocking, cancellable timer future.
///
/// Durations are rounded up to milliseconds and must not exceed the module's
/// configured async-task timeout. A zero duration completes immediately.
pub fn sleep(duration: Duration) -> Sleep {
    Sleep {
        duration,
        #[cfg(target_arch = "wasm32")]
        timer_id: None,
    }
}

impl Future for Sleep {
    type Output = SjmlResult<()>;

    fn poll(self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if this.duration.is_zero() {
            return Poll::Ready(Ok(()));
        }

        #[cfg(target_arch = "wasm32")]
        {
            if this.timer_id.is_none() {
                let mut milliseconds = this.duration.as_millis();
                if this.duration.subsec_nanos() % 1_000_000 != 0 {
                    milliseconds += 1;
                }
                let Ok(milliseconds) = i64::try_from(milliseconds) else {
                    return Poll::Ready(Err(SjmlError::new(
                        ErrorCode::InvalidArgument,
                        "guest timer duration exceeds the supported range",
                    )));
                };
                match host_timer_start(milliseconds) {
                    Ok(timer_id) => this.timer_id = Some(timer_id),
                    Err(error) => return Poll::Ready(Err(error)),
                }
            }

            let timer_id = this.timer_id.expect("timer id was initialized");
            match host_timer_poll(timer_id) {
                Ok(false) => Poll::Pending,
                Ok(true) => {
                    this.timer_id = None;
                    Poll::Ready(Ok(()))
                }
                Err(error) => Poll::Ready(Err(error)),
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            Poll::Ready(Err(SjmlError::new(
                ErrorCode::TaskRejected,
                "guest timers require the wasm32-unknown-unknown target",
            )))
        }
    }
}

impl Drop for Sleep {
    fn drop(&mut self) {
        #[cfg(target_arch = "wasm32")]
        if let Some(timer_id) = self.timer_id.take() {
            // SAFETY: timer cancellation is a fixed numeric import in ABI v1.
            let _ = unsafe { imported_timer_cancel(timer_id) };
        }
    }
}

/// Requests cancellation of one task owned by this guest module.
///
/// The host queues the future's drop on its configured callback lane. A return
/// value of `false` means the id was already complete, cancelled, or unknown;
/// it does not mean the task was running when this request began.
#[cfg(target_arch = "wasm32")]
pub fn cancel(id: GuestTaskId) -> SjmlResult<bool> {
    host_cancel(id)
}

/// Native targets do not have the SJML host scheduler.
#[cfg(not(target_arch = "wasm32"))]
pub fn cancel(_id: GuestTaskId) -> SjmlResult<bool> {
    Err(SjmlError::new(
        ErrorCode::TaskRejected,
        "guest task cancellation requires the wasm32-unknown-unknown target",
    ))
}

#[cfg(any(target_arch = "wasm32", test))]
type LocalFuture = Pin<Box<dyn Future<Output = ()> + 'static>>;
#[cfg(any(target_arch = "wasm32", test))]
type WakeBridge = Rc<dyn Fn(GuestTaskId) -> SjmlResult<()>>;

#[cfg(any(target_arch = "wasm32", test))]
struct LocalExecutor {
    next_id: Option<u32>,
    tasks: HashMap<u32, LocalFuture>,
    wake: WakeBridge,
}

#[cfg(any(target_arch = "wasm32", test))]
impl LocalExecutor {
    fn new(wake: WakeBridge) -> Self {
        Self {
            next_id: Some(1),
            tasks: HashMap::new(),
            wake,
        }
    }

    fn insert(&mut self, future: LocalFuture) -> SjmlResult<GuestTaskId> {
        let id = self.next_id.ok_or_else(|| {
            SjmlError::new(
                ErrorCode::TaskLimitReached,
                "guest task id space is exhausted",
            )
        })?;
        self.next_id = id.checked_add(1);
        self.tasks.insert(id, future);
        GuestTaskId::new(id)
    }

    fn take(&mut self, id: GuestTaskId) -> Option<LocalFuture> {
        self.tasks.remove(&id.get())
    }

    fn restore_pending(&mut self, id: GuestTaskId, future: LocalFuture) -> bool {
        if self.tasks.contains_key(&id.get()) {
            return false;
        }
        self.tasks.insert(id.get(), future);
        true
    }

    fn cancel(&mut self, id: GuestTaskId) -> bool {
        self.tasks.remove(&id.get()).is_some()
    }
}

#[cfg(any(target_arch = "wasm32", test))]
struct TaskWake {
    id: GuestTaskId,
    bridge: WakeBridge,
}

#[cfg(any(target_arch = "wasm32", test))]
impl TaskWake {
    fn request(&self) {
        let _ = (self.bridge)(self.id);
    }
}

#[cfg(any(target_arch = "wasm32", test))]
const RAW_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
    clone_raw_waker,
    wake_raw_waker,
    wake_by_ref_raw_waker,
    drop_raw_waker,
);

#[cfg(any(target_arch = "wasm32", test))]
fn task_waker(id: GuestTaskId, bridge: WakeBridge) -> Waker {
    let data = Rc::into_raw(Rc::new(TaskWake { id, bridge }));
    let raw = RawWaker::new(data.cast(), &RAW_WAKER_VTABLE);
    // SAFETY: every vtable entry below observes the Rc ownership rules for this pointer.
    unsafe { Waker::from_raw(raw) }
}

#[cfg(any(target_arch = "wasm32", test))]
unsafe fn clone_raw_waker(data: *const ()) -> RawWaker {
    // SAFETY: `data` was produced by `Rc::into_raw` in `task_waker` or this vtable.
    let wake = unsafe { Rc::<TaskWake>::from_raw(data.cast()) };
    let clone = Rc::clone(&wake);
    let _ = Rc::into_raw(wake);
    RawWaker::new(Rc::into_raw(clone).cast(), &RAW_WAKER_VTABLE)
}

#[cfg(any(target_arch = "wasm32", test))]
unsafe fn wake_raw_waker(data: *const ()) {
    // SAFETY: this wake consumes exactly one Rc reference owned by the RawWaker.
    let wake = unsafe { Rc::<TaskWake>::from_raw(data.cast()) };
    wake.request();
}

#[cfg(any(target_arch = "wasm32", test))]
unsafe fn wake_by_ref_raw_waker(data: *const ()) {
    // SAFETY: the temporary Rc is restored to raw form so the caller retains ownership.
    let wake = unsafe { Rc::<TaskWake>::from_raw(data.cast()) };
    wake.request();
    let _ = Rc::into_raw(wake);
}

#[cfg(any(target_arch = "wasm32", test))]
unsafe fn drop_raw_waker(data: *const ()) {
    // SAFETY: this drops exactly the Rc reference owned by the RawWaker.
    drop(unsafe { Rc::<TaskWake>::from_raw(data.cast()) });
}

#[cfg(any(target_arch = "wasm32", test))]
fn poll_future(id: GuestTaskId, future: &mut LocalFuture, wake: WakeBridge) -> Poll<()> {
    let waker = task_waker(id, wake);
    let mut context = Context::from_waker(&waker);
    future.as_mut().poll(&mut context)
}

/// Starts a local guest future and asks the host to schedule its initial poll.
///
/// Futures may borrow neither guest stack frames nor temporary input buffers
/// across an await. They are owned by the calling WASM instance until ready,
/// cancelled, timed out, or the module is shut down. The runtime polls them on
/// the main-thread lane for the module's execution context.
#[cfg(target_arch = "wasm32")]
pub fn spawn<F>(future: F) -> SjmlResult<GuestTaskId>
where
    F: Future<Output = ()> + 'static,
{
    let (id, wake) = EXECUTOR.with(|executor| {
        let mut executor = executor.borrow_mut();
        let id = executor.insert(Box::pin(future))?;
        Ok::<_, SjmlError>((id, Rc::clone(&executor.wake)))
    })?;
    if let Err(error) = wake(id) {
        EXECUTOR.with(|executor| {
            executor.borrow_mut().cancel(id);
        });
        return Err(error);
    }
    Ok(id)
}

/// Native host imports are unavailable in unit tests and non-WASM applications.
#[cfg(not(target_arch = "wasm32"))]
pub fn spawn<F>(_future: F) -> SjmlResult<GuestTaskId>
where
    F: Future<Output = ()> + 'static,
{
    Err(SjmlError::new(
        ErrorCode::TaskRejected,
        "guest task spawning requires the wasm32-unknown-unknown target",
    ))
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static EXECUTOR: std::cell::RefCell<LocalExecutor> = std::cell::RefCell::new(
        LocalExecutor::new(Rc::new(host_wake))
    );
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_async_v1")]
extern "C" {
    #[link_name = "wake"]
    fn imported_wake(task_id: i32) -> i32;
    #[link_name = "cancel"]
    fn imported_cancel(task_id: i32) -> i32;
    #[link_name = "timer_start"]
    fn imported_timer_start(delay_millis: i64) -> i32;
    #[link_name = "timer_poll"]
    fn imported_timer_poll(timer_id: i32) -> i32;
    #[link_name = "timer_cancel"]
    fn imported_timer_cancel(timer_id: i32) -> i32;
}

#[cfg(target_arch = "wasm32")]
fn host_wake(id: GuestTaskId) -> SjmlResult<()> {
    // SAFETY: this is the fixed numeric wake import defined by ABI v1.
    let status = unsafe { imported_wake(id.get() as i32) };
    if status == 0 {
        Ok(())
    } else {
        let code = status.unsigned_abs();
        Err(SjmlError::from_abi_code(
            code,
            "host rejected the guest task wake request",
        ))
    }
}

#[cfg(target_arch = "wasm32")]
fn host_cancel(id: GuestTaskId) -> SjmlResult<bool> {
    // SAFETY: this is the fixed numeric cancellation import defined by ABI v1.
    let status = unsafe { imported_cancel(id.get() as i32) };
    match status {
        0 => Ok(false),
        1 => Ok(true),
        status if status < 0 => {
            let code = status.unsigned_abs();
            Err(SjmlError::from_abi_code(
                code,
                "host rejected the guest task cancellation request",
            ))
        }
        _ => Err(SjmlError::new(
            ErrorCode::UnknownErrorCode,
            "host returned an unsupported guest task cancellation status",
        )),
    }
}

#[cfg(target_arch = "wasm32")]
fn host_timer_start(delay_millis: i64) -> SjmlResult<i32> {
    // SAFETY: this is the fixed numeric timer-start import defined by ABI v1.
    let status = unsafe { imported_timer_start(delay_millis) };
    if status > 0 {
        Ok(status)
    } else {
        Err(SjmlError::from_abi_code(
            status.unsigned_abs(),
            "host rejected the guest timer request",
        ))
    }
}

#[cfg(target_arch = "wasm32")]
fn host_timer_poll(timer_id: i32) -> SjmlResult<bool> {
    // SAFETY: this is the fixed numeric timer-poll import defined by ABI v1.
    let status = unsafe { imported_timer_poll(timer_id) };
    match status {
        0 => Ok(false),
        1 => Ok(true),
        status if status < 0 => Err(SjmlError::from_abi_code(
            status.unsigned_abs(),
            "host rejected the guest timer poll",
        )),
        _ => Err(SjmlError::new(
            ErrorCode::UnknownErrorCode,
            "host returned an unsupported guest timer status",
        )),
    }
}

/// Polls exactly once. The host invokes this export only on the configured lane.
#[cfg(target_arch = "wasm32")]
#[export_name = "sjml_v1_task_poll"]
pub extern "C" fn poll(task_id: i32) -> i32 {
    let id = match GuestTaskId::new(task_id as u32) {
        Ok(id) => id,
        Err(error) => return -(error.abi_code() as i32),
    };
    let taken = EXECUTOR.with(|executor| {
        let mut executor = executor.borrow_mut();
        executor
            .take(id)
            .map(|future| (future, Rc::clone(&executor.wake)))
    });
    let Some((mut future, wake)) = taken else {
        return -(ErrorCode::InvalidHandle.abi_code() as i32);
    };
    match poll_future(id, &mut future, wake) {
        Poll::Pending => {
            let restored =
                EXECUTOR.with(|executor| executor.borrow_mut().restore_pending(id, future));
            if restored {
                0
            } else {
                -(ErrorCode::StaleHandle.abi_code() as i32)
            }
        }
        Poll::Ready(()) => 1,
    }
}

/// Drops one future promptly on the lane selected by the host.
#[cfg(target_arch = "wasm32")]
#[export_name = "sjml_v1_task_cancel"]
pub extern "C" fn cancel_export(task_id: i32) -> i32 {
    let id = match GuestTaskId::new(task_id as u32) {
        Ok(id) => id,
        Err(_) => return 0,
    };
    EXECUTOR.with(|executor| i32::from(executor.borrow_mut().cancel(id)))
}

#[cfg(test)]
mod tests {
    use super::{cancel, poll_future, GuestTaskId, LocalExecutor, LocalFuture, WakeBridge};
    use crate::ErrorCode;
    use core::future::Future;
    use core::pin::Pin;
    use core::task::{Context, Poll, Waker};
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;
    use std::time::Duration;

    struct WakeThenReady {
        polls: Rc<Cell<u32>>,
        captured: Rc<RefCell<Option<Waker>>>,
        wake_during_first_poll: bool,
    }

    impl Future for WakeThenReady {
        type Output = ();

        fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            let poll = this.polls.get() + 1;
            this.polls.set(poll);
            if poll == 1 {
                *this.captured.borrow_mut() = Some(context.waker().clone());
                if this.wake_during_first_poll {
                    context.waker().wake_by_ref();
                }
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        }
    }

    struct DropSignal(Rc<Cell<bool>>);

    impl Future for DropSignal {
        type Output = ();

        fn poll(self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<Self::Output> {
            Poll::Pending
        }
    }

    impl Drop for DropSignal {
        fn drop(&mut self) {
            self.0.set(true);
        }
    }

    fn executor() -> (LocalExecutor, Rc<RefCell<Vec<GuestTaskId>>>) {
        let wakes = Rc::new(RefCell::new(Vec::new()));
        let sink = Rc::clone(&wakes);
        let bridge: WakeBridge = Rc::new(move |id| {
            sink.borrow_mut().push(id);
            Ok(())
        });
        (LocalExecutor::new(bridge), wakes)
    }

    fn poll_once(executor: &mut LocalExecutor, id: GuestTaskId) -> Option<bool> {
        let mut future: LocalFuture = executor.take(id)?;
        let result = poll_future(id, &mut future, Rc::clone(&executor.wake));
        match result {
            Poll::Pending => {
                assert!(executor.restore_pending(id, future));
                Some(false)
            }
            Poll::Ready(()) => Some(true),
        }
    }

    #[test]
    fn a_stored_waker_schedules_a_later_poll_and_ready_drops_the_future() {
        let (mut executor, wakes) = executor();
        let polls = Rc::new(Cell::new(0));
        let captured = Rc::new(RefCell::new(None));
        let id = executor
            .insert(Box::pin(WakeThenReady {
                polls: Rc::clone(&polls),
                captured: Rc::clone(&captured),
                wake_during_first_poll: false,
            }))
            .unwrap();

        assert_eq!(poll_once(&mut executor, id), Some(false));
        captured.borrow().as_ref().unwrap().wake_by_ref();
        assert_eq!(wakes.borrow().as_slice(), &[id]);
        assert_eq!(poll_once(&mut executor, id), Some(true));
        assert_eq!(poll_once(&mut executor, id), None);
        assert_eq!(polls.get(), 2);
    }

    #[test]
    fn wake_during_poll_is_recorded_without_reentrant_executor_borrowing() {
        let (mut executor, wakes) = executor();
        let id = executor
            .insert(Box::pin(WakeThenReady {
                polls: Rc::new(Cell::new(0)),
                captured: Rc::new(RefCell::new(None)),
                wake_during_first_poll: true,
            }))
            .unwrap();

        assert_eq!(poll_once(&mut executor, id), Some(false));
        assert_eq!(wakes.borrow().as_slice(), &[id]);
        assert_eq!(poll_once(&mut executor, id), Some(true));
    }

    #[test]
    fn cancellation_drops_the_owned_future_and_unknown_ids_are_rejected() {
        let (mut executor, _) = executor();
        let dropped = Rc::new(Cell::new(false));
        let id = executor
            .insert(Box::pin(DropSignal(Rc::clone(&dropped))))
            .unwrap();

        assert!(executor.cancel(id));
        assert!(dropped.get());
        assert!(!executor.cancel(id));
        let error = GuestTaskId::new(0).unwrap_err();
        assert_eq!(error.code(), ErrorCode::InvalidHandle);
    }

    #[test]
    fn public_cancellation_reports_that_native_host_imports_are_unavailable() {
        let id = GuestTaskId::new(1).unwrap();
        assert_eq!(cancel(id).unwrap_err().code(), ErrorCode::TaskRejected);
    }

    #[test]
    fn zero_sleep_completes_natively_and_nonzero_sleep_fails_closed() {
        let waker = Waker::noop();
        let mut context = Context::from_waker(waker);
        let mut immediate = Box::pin(super::sleep(Duration::ZERO));
        assert!(matches!(
            immediate.as_mut().poll(&mut context),
            Poll::Ready(Ok(()))
        ));

        let mut delayed = Box::pin(super::sleep(Duration::from_millis(1)));
        let Poll::Ready(Err(error)) = delayed.as_mut().poll(&mut context) else {
            panic!("native guest timers must fail closed");
        };
        assert_eq!(error.code(), ErrorCode::TaskRejected);
    }

    #[test]
    fn yield_now_queues_one_later_poll_before_completing() {
        let (mut executor, wakes) = executor();
        let id = executor.insert(Box::pin(super::yield_now())).unwrap();

        assert_eq!(poll_once(&mut executor, id), Some(false));
        assert_eq!(wakes.borrow().as_slice(), &[id]);
        assert_eq!(poll_once(&mut executor, id), Some(true));
        assert_eq!(poll_once(&mut executor, id), None);
    }
}
