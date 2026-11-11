//! Isolated, bounded host worker computations.

#[cfg(target_arch = "wasm32")]
use crate::from_bytes;
use crate::{to_bytes, ErrorCode, SjmlError, SjmlResult, StableDecode, StableEncode};
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};

/// Maximum encoded input or output accepted by one worker computation.
pub const MAX_COMPUTE_BYTES: usize = 256 * 1024;

/// Starts a named `#[compute]` callback in a separate, capability-denied WASM instance.
///
/// Input is serialized before this function returns. The worker receives only those
/// stable-encoded bytes and cannot access the caller's memory or host imports.
pub fn spawn_compute<I, O>(callback: &'static str, input: &I) -> SjmlResult<ComputeFuture<O>>
where
    I: StableEncode + ?Sized,
    O: StableDecode,
{
    if !valid_callback_name(callback) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "compute callback name must be a lowercase ASCII identifier of at most 128 bytes",
        ));
    }
    let input = to_bytes(input)?;
    if input.len() > MAX_COMPUTE_BYTES {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "compute input exceeds the 256 KiB worker limit",
        ));
    }
    Ok(ComputeFuture {
        callback: callback.as_bytes().to_vec(),
        input,
        output: vec![0; MAX_COMPUTE_BYTES],
        operation_id: None,
        completed: false,
        output_type: PhantomData,
    })
}

fn valid_callback_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && matches!(value.as_bytes()[0], b'a'..=b'z' | b'0'..=b'9')
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

/// Future returned by [`spawn_compute`]. Dropping it cancels the host operation
/// and asks the worker executor to interrupt a running callback when possible.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub struct ComputeFuture<O> {
    callback: Vec<u8>,
    input: Vec<u8>,
    output: Vec<u8>,
    operation_id: Option<i32>,
    completed: bool,
    output_type: PhantomData<fn() -> O>,
}

impl<O: StableDecode> Future for ComputeFuture<O> {
    type Output = SjmlResult<O>;

    fn poll(self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if this.completed {
            return Poll::Ready(Err(SjmlError::new(
                ErrorCode::InvalidHandle,
                "compute future was polled after completion",
            )));
        }

        #[cfg(target_arch = "wasm32")]
        {
            if this.operation_id.is_none() {
                // SAFETY: the host copies both spans before returning and bounds-checks them.
                let operation_id = unsafe {
                    imported_compute_start(
                        this.callback.as_ptr() as usize as u32 as i32,
                        this.callback.len() as u32 as i32,
                        this.input.as_ptr() as usize as u32 as i32,
                        this.input.len() as u32 as i32,
                    )
                };
                if operation_id <= 0 {
                    this.completed = true;
                    return Poll::Ready(Err(SjmlError::from_abi_code(
                        operation_id.unsigned_abs(),
                        "host rejected the worker computation",
                    )));
                }
                this.operation_id = Some(operation_id);
            }

            let operation_id = this
                .operation_id
                .expect("compute operation id was initialized");
            // SAFETY: the host writes no more than the supplied output capacity.
            let packed = unsafe {
                imported_compute_poll(
                    operation_id,
                    this.output.as_mut_ptr() as usize as u32 as i32,
                    this.output.len() as u32 as i32,
                )
            } as u64;
            let status = packed as u32 as i32;
            let output_length = (packed >> 32) as u32 as usize;
            if status == 0 {
                return Poll::Pending;
            }
            this.operation_id = None;
            this.completed = true;
            if status != 1 {
                return Poll::Ready(Err(SjmlError::from_abi_code(
                    status.unsigned_abs(),
                    "worker computation failed",
                )));
            }
            if output_length > this.output.len() {
                return Poll::Ready(Err(SjmlError::new(
                    ErrorCode::PayloadTooLarge,
                    "worker output exceeds the guest-provided output buffer",
                )));
            }
            Poll::Ready(from_bytes::<O>(&this.output[..output_length]))
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            this.completed = true;
            Poll::Ready(Err(SjmlError::new(
                ErrorCode::TaskRejected,
                "worker computations require the wasm32-unknown-unknown target",
            )))
        }
    }
}

impl<O> Drop for ComputeFuture<O> {
    fn drop(&mut self) {
        #[cfg(target_arch = "wasm32")]
        if !self.completed {
            if let Some(operation_id) = self.operation_id.take() {
                // SAFETY: cancellation accepts only an opaque operation id.
                let _ = unsafe { imported_compute_cancel(operation_id) };
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_async_v1")]
extern "C" {
    #[link_name = "compute_start"]
    fn imported_compute_start(
        callback_pointer: i32,
        callback_length: i32,
        input_pointer: i32,
        input_length: i32,
    ) -> i32;
    #[link_name = "compute_poll"]
    fn imported_compute_poll(operation_id: i32, output_pointer: i32, output_capacity: i32) -> i64;
    #[link_name = "compute_cancel"]
    fn imported_compute_cancel(operation_id: i32) -> i32;
}

#[cfg(test)]
mod tests {
    use super::{spawn_compute, MAX_COMPUTE_BYTES};
    use crate::{ErrorCode, StableDecode, StableEncode};
    use core::future::Future;
    use core::pin::Pin;
    use core::task::{Context, Poll, Waker};

    #[derive(Debug, PartialEq, StableEncode, StableDecode)]
    #[sjml(version = 1)]
    struct Output {
        value: u32,
    }

    #[test]
    fn worker_inputs_are_serialized_and_native_execution_fails_closed() {
        let mut future = Box::pin(spawn_compute::<u32, Output>("demo.double", &3).unwrap());
        let mut context = Context::from_waker(Waker::noop());
        let Poll::Ready(Err(error)) = Pin::as_mut(&mut future).poll(&mut context) else {
            panic!("native worker execution must fail closed");
        };
        assert_eq!(error.code(), ErrorCode::TaskRejected);
    }

    #[test]
    fn invalid_callback_names_and_oversized_inputs_are_rejected() {
        assert_eq!(
            spawn_compute::<u32, Output>("Bad Name", &1)
                .err()
                .unwrap()
                .code(),
            ErrorCode::InvalidArgument
        );
        let input = vec![0_u8; MAX_COMPUTE_BYTES];
        let error = spawn_compute::<Vec<u8>, Output>("demo.large", &input)
            .err()
            .unwrap();
        assert_eq!(error.code(), ErrorCode::PayloadTooLarge);
    }
}
