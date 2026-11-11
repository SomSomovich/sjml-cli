//! Exclusive runtime control of the native client chat pane in logical GUI units.

#[cfg(any(target_arch = "wasm32", test))]
use crate::{from_bytes, StableDecode, StableEncode};
use crate::{ErrorCode, SjmlError, SjmlResult};

/// A finite native chat rectangle. The host clamps it to the viewport and minimum size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChatLayout {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl ChatLayout {
    /// Checks finite coordinates and finite, strictly positive dimensions.
    /// Negative positions are allowed as requests; the host returns the clamped rectangle.
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> SjmlResult<Self> {
        if !x.is_finite()
            || !y.is_finite()
            || !width.is_finite()
            || !height.is_finite()
            || width <= 0.0
            || height <= 0.0
        {
            return Err(invalid(
                "chat layout requires finite coordinates and positive finite dimensions",
            ));
        }
        Ok(Self {
            x,
            y,
            width,
            height,
        })
    }
    pub const fn x(self) -> f32 {
        self.x
    }
    pub const fn y(self) -> f32 {
        self.y
    }
    pub const fn width(self) -> f32 {
        self.width
    }
    pub const fn height(self) -> f32 {
        self.height
    }
}

#[derive(Debug)]
enum ChatLayoutTag {}
impl crate::HandleKind for ChatLayoutTag {
    const KIND_CODE: u32 = 13;
}

/// Opaque, module-owned token for the exclusive chat layout controller.
/// Copies expire together on unsubscription or runtime close.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChatLayoutHandle(crate::Handle<ChatLayoutTag>);

/// Native ChatScreen lifecycle and bounded logical GUI pointer input.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ChatScreenEvent {
    Opened {
        viewport_width: f32,
        viewport_height: f32,
        layout: ChatLayout,
    },
    Closed {
        viewport_width: f32,
        viewport_height: f32,
        layout: ChatLayout,
    },
    ViewportResized {
        viewport_width: f32,
        viewport_height: f32,
        layout: ChatLayout,
    },
    MousePressed {
        mouse_x: f32,
        mouse_y: f32,
        button: u8,
    },
    MouseDragged {
        mouse_x: f32,
        mouse_y: f32,
        button: u8,
        delta_x: f32,
        delta_y: f32,
    },
    MouseReleased {
        mouse_x: f32,
        mouse_y: f32,
        button: u8,
    },
}

#[cfg(any(target_arch = "wasm32", test))]
impl ChatScreenEvent {
    fn consumable(self) -> bool {
        matches!(
            self,
            Self::MousePressed { button: 0, .. }
                | Self::MouseDragged { button: 0, .. }
                | Self::MouseReleased { button: 0, .. }
        )
    }
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_client_chat_v1")]
extern "C" {
    #[link_name = "subscribe"]
    fn imported_subscribe(callback_id: i64, output_handle: i32) -> i32;
    #[link_name = "unsubscribe"]
    fn imported_unsubscribe(handle_pointer: i32, handle_length: i32) -> i32;
    #[link_name = "layout"]
    fn imported_layout(handle_pointer: i32, handle_length: i32, output_layout: i32) -> i32;
    #[link_name = "set_layout"]
    fn imported_set_layout(
        handle_pointer: i32,
        handle_length: i32,
        layout_pointer: i32,
        layout_length: i32,
        output_layout: i32,
    ) -> i32;
}

#[cfg(any(target_arch = "wasm32", test))]
type Callback =
    std::rc::Rc<std::cell::RefCell<Box<dyn FnMut(ChatScreenEvent) -> SjmlResult<bool>>>>;
#[cfg(any(target_arch = "wasm32", test))]
struct CallbackEntry {
    callback: Callback,
    handle: Option<ChatLayoutHandle>,
}
#[cfg(any(target_arch = "wasm32", test))]
struct Callbacks {
    next: Option<u64>,
    entries: std::collections::BTreeMap<u64, CallbackEntry>,
}
#[cfg(any(target_arch = "wasm32", test))]
impl Default for Callbacks {
    fn default() -> Self {
        Self {
            next: Some(1),
            entries: Default::default(),
        }
    }
}
#[cfg(any(target_arch = "wasm32", test))]
impl Callbacks {
    fn insert<F>(&mut self, callback: F) -> SjmlResult<u64>
    where
        F: FnMut(ChatScreenEvent) -> SjmlResult<bool> + 'static,
    {
        if !self.entries.is_empty() {
            return Err(SjmlError::new(
                ErrorCode::RegistrationLimitReached,
                "chat layout controller is already subscribed",
            ));
        }
        let id = self.next.ok_or_else(|| {
            SjmlError::new(
                ErrorCode::RegistrationLimitReached,
                "chat callback identities exhausted",
            )
        })?;
        self.next = id.checked_add(1).filter(|next| *next <= i64::MAX as u64);
        self.entries.insert(
            id,
            CallbackEntry {
                callback: std::rc::Rc::new(std::cell::RefCell::new(Box::new(callback))),
                handle: None,
            },
        );
        Ok(id)
    }
    fn attach(&mut self, id: u64, handle: ChatLayoutHandle) -> SjmlResult<()> {
        self.entries.get_mut(&id).ok_or_else(stale)?.handle = Some(handle);
        Ok(())
    }
    fn get(&self, id: u64) -> SjmlResult<Callback> {
        self.entries
            .get(&id)
            .filter(|entry| entry.handle.is_some())
            .map(|entry| entry.callback.clone())
            .ok_or_else(stale)
    }
    fn id(&self, handle: ChatLayoutHandle) -> SjmlResult<u64> {
        self.entries
            .iter()
            .find_map(|(&id, entry)| (entry.handle == Some(handle)).then_some(id))
            .ok_or_else(stale)
    }
    fn remove(&mut self, handle: ChatLayoutHandle) {
        self.entries.retain(|_, entry| entry.handle != Some(handle));
    }
}
#[cfg(any(target_arch = "wasm32", test))]
thread_local! {
    static CALLBACKS: std::cell::RefCell<Callbacks> = std::cell::RefCell::new(Callbacks::default());
    static DISPATCHING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Registers the exclusive controller during client runtime initialization.
/// Requires a requested and granted `client.chat.modify` capability. Return `true`
/// only when handling a left-button move/resize grip; other events cannot be consumed.
pub fn subscribe<F>(callback: F) -> SjmlResult<ChatLayoutHandle>
where
    F: FnMut(ChatScreenEvent) -> SjmlResult<bool> + 'static,
{
    #[cfg(target_arch = "wasm32")]
    {
        let id = CALLBACKS.with(|state| state.borrow_mut().insert(callback))?;
        let mut output = [0u8; 24];
        // SAFETY: the writable output buffer lives for the complete synchronous import.
        let status = unsafe { imported_subscribe(id as i64, output.as_mut_ptr() as i32) };
        let result = host_status(status)
            .and_then(|()| from_bytes(&output))
            .map(ChatLayoutHandle);
        match result {
            Ok(handle) => {
                CALLBACKS.with(|state| state.borrow_mut().attach(id, handle))?;
                Ok(handle)
            }
            Err(error) => {
                CALLBACKS.with(|state| state.borrow_mut().entries.remove(&id));
                Err(error)
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = callback;
        Err(unsupported())
    }
}

/// Revokes the controller and its callback route. Layout control returns to vanilla.
pub fn unsubscribe(handle: ChatLayoutHandle) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        CALLBACKS.with(|state| state.borrow().id(handle))?;
        let bytes = crate::to_bytes(&handle.0)?;
        // SAFETY: exact stable handle bytes remain live during the synchronous host call.
        let status = unsafe { imported_unsubscribe(bytes.as_ptr() as i32, bytes.len() as i32) };
        let result = host_status(status);
        if result.is_ok() || is_expired(status) {
            CALLBACKS.with(|state| state.borrow_mut().remove(handle));
        }
        result
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported())
    }
}

/// Reads the current host-clamped rectangle for the active controller.
pub fn layout(handle: ChatLayoutHandle) -> SjmlResult<ChatLayout> {
    #[cfg(target_arch = "wasm32")]
    {
        query_layout(handle, None)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported())
    }
}

/// Applies a rectangle and returns the authoritative host-clamped result.
/// Minimum size is 120 by 56 GUI units, reduced to fit smaller viewports.
pub fn set_layout(handle: ChatLayoutHandle, layout: ChatLayout) -> SjmlResult<ChatLayout> {
    #[cfg(target_arch = "wasm32")]
    {
        query_layout(handle, Some(layout))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, layout);
        Err(unsupported())
    }
}

#[cfg(target_arch = "wasm32")]
fn query_layout(handle: ChatLayoutHandle, requested: Option<ChatLayout>) -> SjmlResult<ChatLayout> {
    CALLBACKS.with(|state| state.borrow().id(handle))?;
    let bytes = crate::to_bytes(&handle.0)?;
    let mut output = [0u8; 16];
    let status = if let Some(layout) = requested {
        let layout_bytes = crate::to_bytes(&WireLayout(layout))?;
        // SAFETY: all buffers are disjoint, initialized, and live for this synchronous call.
        unsafe {
            imported_set_layout(
                bytes.as_ptr() as i32,
                bytes.len() as i32,
                layout_bytes.as_ptr() as i32,
                layout_bytes.len() as i32,
                output.as_mut_ptr() as i32,
            )
        }
    } else {
        // SAFETY: stable handle and writable output remain live during the synchronous call.
        unsafe {
            imported_layout(
                bytes.as_ptr() as i32,
                bytes.len() as i32,
                output.as_mut_ptr() as i32,
            )
        }
    };
    if is_expired(status) {
        CALLBACKS.with(|state| state.borrow_mut().remove(handle));
    }
    host_status(status)?;
    Ok(from_bytes::<WireLayout>(&output)?.0)
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug)]
struct WireLayout(ChatLayout);
#[cfg(any(target_arch = "wasm32", test))]
impl StableEncode for WireLayout {
    fn encode(&self, encoder: &mut crate::Encoder) -> SjmlResult<()> {
        encoder.write_f32(self.0.x)?;
        encoder.write_f32(self.0.y)?;
        encoder.write_f32(self.0.width)?;
        encoder.write_f32(self.0.height)
    }
}
#[cfg(any(target_arch = "wasm32", test))]
impl StableDecode for WireLayout {
    fn decode(decoder: &mut crate::Decoder<'_>) -> SjmlResult<Self> {
        ChatLayout::new(
            decoder.read_f32()?,
            decoder.read_f32()?,
            decoder.read_f32()?,
            decoder.read_f32()?,
        )
        .map(Self)
    }
}
#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug)]
struct Delivery {
    callback_id: u64,
    event: ChatScreenEvent,
}
#[cfg(any(target_arch = "wasm32", test))]
impl StableDecode for Delivery {
    fn decode(decoder: &mut crate::Decoder<'_>) -> SjmlResult<Self> {
        if decoder.read_u32()? != 1 {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                "unsupported client chat event schema",
            ));
        }
        let callback_id = decoder.read_u64()?;
        if callback_id == 0 || callback_id > i64::MAX as u64 {
            return Err(invalid("invalid client chat callback identity"));
        }
        let tag = decoder.read_u8()?;
        let event = match tag {
            0..=2 => {
                let viewport_width = decoder.read_f32()?;
                let viewport_height = decoder.read_f32()?;
                for value in [viewport_width, viewport_height] {
                    coordinate(value)?;
                    if value <= 0.0 {
                        return Err(invalid("chat viewport dimensions must be positive"));
                    }
                }
                let layout = WireLayout::decode(decoder)?.0;
                match tag {
                    0 => ChatScreenEvent::Opened {
                        viewport_width,
                        viewport_height,
                        layout,
                    },
                    1 => ChatScreenEvent::Closed {
                        viewport_width,
                        viewport_height,
                        layout,
                    },
                    _ => ChatScreenEvent::ViewportResized {
                        viewport_width,
                        viewport_height,
                        layout,
                    },
                }
            }
            3..=5 => {
                let mouse_x = decoder.read_f32()?;
                let mouse_y = decoder.read_f32()?;
                coordinate(mouse_x)?;
                coordinate(mouse_y)?;
                let button = decoder.read_u8()?;
                if button > 31 {
                    return Err(invalid("chat mouse button exceeds 31"));
                }
                match tag {
                    3 => ChatScreenEvent::MousePressed {
                        mouse_x,
                        mouse_y,
                        button,
                    },
                    5 => ChatScreenEvent::MouseReleased {
                        mouse_x,
                        mouse_y,
                        button,
                    },
                    _ => {
                        let delta_x = decoder.read_f32()?;
                        let delta_y = decoder.read_f32()?;
                        coordinate(delta_x)?;
                        coordinate(delta_y)?;
                        ChatScreenEvent::MouseDragged {
                            mouse_x,
                            mouse_y,
                            button,
                            delta_x,
                            delta_y,
                        }
                    }
                }
            }
            _ => return Err(invalid("unknown client chat event tag")),
        };
        Ok(Self { callback_id, event })
    }
}
#[cfg(any(target_arch = "wasm32", test))]
fn coordinate(value: f32) -> SjmlResult<()> {
    if !value.is_finite() || value.abs() > 1_048_576.0 {
        return Err(invalid(
            "chat pointer or viewport exceeds finite coordinate bounds",
        ));
    }
    Ok(())
}
#[cfg(any(target_arch = "wasm32", test))]
struct DispatchGuard;
#[cfg(any(target_arch = "wasm32", test))]
impl Drop for DispatchGuard {
    fn drop(&mut self) {
        DISPATCHING.with(|state| state.set(false));
    }
}
#[cfg(any(target_arch = "wasm32", test))]
fn dispatch_delivery(delivery: Delivery) -> SjmlResult<bool> {
    if DISPATCHING.with(|state| state.replace(true)) {
        return Err(SjmlError::new(
            ErrorCode::ServiceBusy,
            "reentrant client chat dispatch is forbidden",
        ));
    }
    let _guard = DispatchGuard;
    let callback = CALLBACKS.with(|state| state.borrow().get(delivery.callback_id))?;
    let mut callback = callback.try_borrow_mut().map_err(|_| {
        SjmlError::new(
            ErrorCode::ServiceBusy,
            "client chat callback is already active",
        )
    })?;
    let consumed = callback(delivery.event)?;
    Ok(consumed && delivery.event.consumable())
}
#[cfg(test)]
fn deliver(bytes: &[u8]) -> SjmlResult<bool> {
    dispatch_delivery(from_bytes(bytes)?)
}

/// Delivers a schema-1 chat event and returns the standard packed ByteSpan response.
/// The success response contains exactly one stable bool byte.
///
/// # Safety
/// The host supplies a live initialized input allocation from `sjml_v1_alloc`, keeps it
/// unmodified until return, then frees input and response with the paired ABI exports.
#[cfg(target_arch = "wasm32")]
#[export_name = "sjml_v1_client_chat_dispatch"]
pub unsafe extern "C" fn dispatch(pointer: u32, length: u32) -> u64 {
    // SAFETY: the caller guarantees the initialized input allocation's live span.
    unsafe { crate::abi::dispatch_payload(pointer, length, dispatch_delivery) }
}
#[cfg(any(target_arch = "wasm32", test))]
fn stale() -> SjmlError {
    SjmlError::new(
        ErrorCode::StaleHandle,
        "chat controller is not active in this module runtime",
    )
}
#[cfg(target_arch = "wasm32")]
fn host_status(status: i32) -> SjmlResult<()> {
    if status == 0 {
        Ok(())
    } else {
        Err(SjmlError::from_abi_code(
            status as u32,
            "host rejected client chat operation",
        ))
    }
}
#[cfg(target_arch = "wasm32")]
fn is_expired(status: i32) -> bool {
    status == ErrorCode::StaleHandle.abi_code() as i32
        || status == ErrorCode::HandleTableClosed.abi_code() as i32
}
fn invalid(message: &'static str) -> SjmlError {
    SjmlError::new(ErrorCode::InvalidArgument, message)
}
#[cfg(not(target_arch = "wasm32"))]
fn unsupported() -> SjmlError {
    SjmlError::new(
        ErrorCode::PermissionDenied,
        "client chat requires a wasm32 client runtime",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::Cell, rc::Rc};

    fn packet(tag: u8, body: &[u8]) -> Vec<u8> {
        let mut bytes = vec![1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, tag];
        bytes.extend_from_slice(body);
        bytes
    }
    fn floats(values: &[f32]) -> Vec<u8> {
        values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect()
    }
    fn register<F>(callback: F) -> (u64, ChatLayoutHandle)
    where
        F: FnMut(ChatScreenEvent) -> SjmlResult<bool> + 'static,
    {
        CALLBACKS.with(|state| *state.borrow_mut() = Callbacks::default());
        let id = CALLBACKS
            .with(|state| state.borrow_mut().insert(callback))
            .unwrap();
        let handle = ChatLayoutHandle(crate::Handle::new(1, 2, 3, 4).unwrap());
        CALLBACKS
            .with(|state| state.borrow_mut().attach(id, handle))
            .unwrap();
        (id, handle)
    }
    #[test]
    fn decoder_matches_all_six_wire_tags_and_exact_field_order() {
        for tag in 0..=2 {
            let event = crate::from_bytes::<Delivery>(&packet(
                tag,
                &floats(&[854.0, 480.0, 2.0, 3.0, 240.0, 100.0]),
            ))
            .unwrap()
            .event;
            let layout = ChatLayout::new(2.0, 3.0, 240.0, 100.0).unwrap();
            let expected = match tag {
                0 => ChatScreenEvent::Opened {
                    viewport_width: 854.0,
                    viewport_height: 480.0,
                    layout,
                },
                1 => ChatScreenEvent::Closed {
                    viewport_width: 854.0,
                    viewport_height: 480.0,
                    layout,
                },
                _ => ChatScreenEvent::ViewportResized {
                    viewport_width: 854.0,
                    viewport_height: 480.0,
                    layout,
                },
            };
            assert_eq!(event, expected);
        }
        let mut mouse = floats(&[-4.0, 5.0]);
        mouse.push(0);
        assert_eq!(
            crate::from_bytes::<Delivery>(&packet(3, &mouse))
                .unwrap()
                .event,
            ChatScreenEvent::MousePressed {
                mouse_x: -4.0,
                mouse_y: 5.0,
                button: 0
            }
        );
        assert_eq!(
            crate::from_bytes::<Delivery>(&packet(5, &mouse))
                .unwrap()
                .event,
            ChatScreenEvent::MouseReleased {
                mouse_x: -4.0,
                mouse_y: 5.0,
                button: 0
            }
        );
        mouse.extend(floats(&[1.5, -2.0]));
        assert_eq!(
            crate::from_bytes::<Delivery>(&packet(4, &mouse))
                .unwrap()
                .event,
            ChatScreenEvent::MouseDragged {
                mouse_x: -4.0,
                mouse_y: 5.0,
                button: 0,
                delta_x: 1.5,
                delta_y: -2.0
            }
        );
    }
    #[test]
    fn decoder_rejects_bad_schema_identity_tags_trailing_and_truncated_bytes() {
        let mut body = floats(&[0.0, 0.0]);
        body.push(0);
        let good = packet(3, &body);
        for index in 0..good.len() {
            assert!(crate::from_bytes::<Delivery>(&good[..index]).is_err());
        }
        let mut bytes = good.clone();
        bytes.push(0);
        assert!(crate::from_bytes::<Delivery>(&bytes).is_err());
        let mut bytes = good.clone();
        bytes[0] = 2;
        assert_eq!(
            crate::from_bytes::<Delivery>(&bytes).unwrap_err().code(),
            ErrorCode::IncompatibleAbiVersion
        );
        let mut bytes = good.clone();
        bytes[4] = 0;
        assert!(crate::from_bytes::<Delivery>(&bytes).is_err());
        let mut bytes = good;
        bytes[12] = 6;
        assert!(crate::from_bytes::<Delivery>(&bytes).is_err());
    }
    #[test]
    fn decoder_enforces_mouse_delta_button_and_viewport_bounds() {
        for value in [f32::NAN, f32::INFINITY, 1_048_577.0, -1_048_577.0] {
            for index in 0..4 {
                let mut values = [0.0; 4];
                values[index] = value;
                let mut body = floats(&values[..2]);
                body.push(0);
                body.extend(floats(&values[2..]));
                assert!(crate::from_bytes::<Delivery>(&packet(4, &body)).is_err());
            }
        }
        let mut body = floats(&[1_048_576.0, -1_048_576.0]);
        body.push(31);
        assert!(crate::from_bytes::<Delivery>(&packet(3, &body)).is_ok());
        body[8] = 32;
        assert!(crate::from_bytes::<Delivery>(&packet(3, &body)).is_err());
        for value in [0.0, -1.0, f32::NAN, 1_048_577.0] {
            assert!(crate::from_bytes::<Delivery>(&packet(
                0,
                &floats(&[value, 480.0, 0.0, 0.0, 120.0, 56.0])
            ))
            .is_err());
        }
    }
    #[test]
    fn callbacks_are_exclusive_and_expired_routes_stay_stale() {
        let (_, handle) = register(|_| Ok(false));
        assert_eq!(
            CALLBACKS.with(|state| state.borrow().id(handle)).unwrap(),
            1
        );
        assert_eq!(
            CALLBACKS
                .with(|state| state.borrow_mut().insert(|_| Ok(false)))
                .unwrap_err()
                .code(),
            ErrorCode::RegistrationLimitReached
        );
        CALLBACKS.with(|state| state.borrow_mut().remove(handle));
        assert_eq!(
            deliver(&packet(3, &[0, 0, 0, 0, 0, 0, 0, 0, 0]))
                .unwrap_err()
                .code(),
            ErrorCode::StaleHandle
        );
        let id = CALLBACKS
            .with(|state| state.borrow_mut().insert(|_| Ok(false)))
            .unwrap();
        assert_eq!(id, 2);
        assert_eq!(
            CALLBACKS
                .with(|state| state.borrow().get(1))
                .err()
                .unwrap()
                .code(),
            ErrorCode::StaleHandle
        );
        assert_eq!(
            CALLBACKS
                .with(|state| state.borrow().get(id))
                .err()
                .unwrap()
                .code(),
            ErrorCode::StaleHandle
        );
    }
    #[test]
    fn callback_identity_exhaustion_never_reuses_expired_routes() {
        let mut callbacks = Callbacks {
            next: Some(i64::MAX as u64),
            entries: Default::default(),
        };
        assert_eq!(callbacks.insert(|_| Ok(false)).unwrap(), i64::MAX as u64);
        callbacks.entries.clear();
        assert_eq!(
            callbacks.insert(|_| Ok(false)).unwrap_err().code(),
            ErrorCode::RegistrationLimitReached
        );
    }
    #[test]
    fn only_left_pointer_events_are_consumable() {
        let count = Rc::new(Cell::new(0));
        let seen = count.clone();
        register(move |_| {
            seen.set(seen.get() + 1);
            Ok(true)
        });
        for button in [0, 1, 31] {
            for tag in [3, 4, 5] {
                let mut body = floats(&[0.0, 0.0]);
                body.push(button);
                if tag == 4 {
                    body.extend(floats(&[0.0, 0.0]));
                }
                assert_eq!(deliver(&packet(tag, &body)).unwrap(), button == 0);
            }
        }
        assert!(!deliver(&packet(1, &floats(&[854.0, 480.0, 0.0, 0.0, 120.0, 56.0]))).unwrap());
        assert_eq!(count.get(), 10);
    }
    #[test]
    fn reentrant_dispatch_returns_busy_and_restores_dispatch_state() {
        register(|_| {
            assert_eq!(
                deliver(&packet(3, &[0; 9])).unwrap_err().code(),
                ErrorCode::ServiceBusy
            );
            Err(SjmlError::new(ErrorCode::InvalidArgument, "callback error"))
        });
        for _ in 0..2 {
            assert_eq!(
                deliver(&packet(3, &[0; 9])).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
        }
        assert!(!DISPATCHING.with(Cell::get));
    }
    #[test]
    fn callback_can_remove_its_route_without_borrow_panics() {
        let handle_cell = Rc::new(Cell::new(None));
        let own_handle = handle_cell.clone();
        let (_, handle) = register(move |_| {
            CALLBACKS.with(|state| state.borrow_mut().remove(own_handle.get().unwrap()));
            Ok(true)
        });
        handle_cell.set(Some(handle));
        assert!(deliver(&packet(3, &[0; 9])).unwrap());
        assert_eq!(
            deliver(&packet(3, &[0; 9])).unwrap_err().code(),
            ErrorCode::StaleHandle
        );
    }
    #[test]
    fn private_layout_codec_is_exact_and_validates_host_output() {
        let layout = ChatLayout::new(2.0, 3.0, 240.0, 100.0).unwrap();
        assert_eq!(
            crate::to_bytes(&WireLayout(layout)).unwrap(),
            vec![0, 0, 0, 64, 0, 0, 64, 64, 0, 0, 112, 67, 0, 0, 200, 66]
        );
        assert_eq!(
            crate::from_bytes::<WireLayout>(&floats(&[2.0, 3.0, 240.0, 100.0]))
                .unwrap()
                .0,
            layout
        );
        assert!(crate::from_bytes::<WireLayout>(&floats(&[0.0, 0.0, 0.0, 100.0])).is_err());
    }
}
