//! Local client chat message observation and display-only filtering.
//!
//! Signed server messages have already passed Minecraft's signature and filter
//! checks before this callback. The guest receives detached plain text only; it
//! cannot alter, resend, or change the server-side message. Hiding a message
//! requires the separately granted `client.chat.filter` capability.

#[cfg(target_arch = "wasm32")]
use crate::abi::{allocate_host_input, deallocate_host_input};
#[cfg(target_arch = "wasm32")]
use crate::{from_bytes, to_bytes};
use crate::{Decoder, StableDecode, StableEncode};
use crate::{ErrorCode, SjmlError, SjmlResult};

/// Maximum number of local client chat subscriptions owned by one module.
pub const MAX_CHAT_MESSAGE_SUBSCRIPTIONS: usize = 128;
/// Maximum UTF-8 bytes exposed for one message; longer messages are truncated at a code point boundary.
pub const MAX_CHAT_MESSAGE_BYTES: usize = 4096;
/// Maximum UTF-8 bytes accepted by [`send`].
pub const MAX_OUTGOING_CHAT_BYTES: usize = 1024;
/// Minecraft's chat input is limited to 256 UTF-16 code units.
pub const MAX_OUTGOING_CHAT_UTF16_UNITS: usize = 256;
#[cfg(any(target_arch = "wasm32", test))]
const MESSAGE_SCHEMA_VERSION: u32 = 1;
#[cfg(target_arch = "wasm32")]
const HANDLE_BYTES: u32 = 24;

/// Origin classification available at the local chat display insertion point.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChatMessageSource {
    /// Minecraft supplied a player-message signature after normal verification/filtering.
    PlayerChat,
    /// Unsigned or system text supplied to the client chat component.
    System,
}

/// Detached, bounded text visible to one local client.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChatMessage {
    source: ChatMessageSource,
    text: String,
    truncated: bool,
}

impl ChatMessage {
    pub fn source(&self) -> ChatMessageSource {
        self.source
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn was_truncated(&self) -> bool {
        self.truncated
    }
}

/// Requested local display behavior. `Hide` is honored only with `client.chat.filter`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChatMessageAction {
    Show,
    Hide,
}

#[derive(Debug)]
enum ChatMessageSubscriptionTag {}
impl crate::HandleKind for ChatMessageSubscriptionTag {
    const KIND_CODE: u32 = 14;
}

/// Runtime-owned subscription token. It becomes stale on unsubscribe or context close.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChatMessageSubscriptionHandle(crate::Handle<ChatMessageSubscriptionTag>);

impl StableEncode for ChatMessageSubscriptionHandle {
    fn encode(&self, encoder: &mut crate::Encoder) -> SjmlResult<()> {
        self.0.encode(encoder)
    }
}

impl StableDecode for ChatMessageSubscriptionHandle {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        crate::Handle::decode(decoder).map(Self)
    }
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_client_chat_messages_v1")]
extern "C" {
    #[link_name = "subscribe"]
    fn imported_subscribe(callback_id: i64, output_handle: i32) -> i32;
    #[link_name = "unsubscribe"]
    fn imported_unsubscribe(handle_pointer: i32, handle_length: i32) -> i32;
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_client_chat_send_v1")]
extern "C" {
    #[link_name = "send"]
    fn imported_send(message_pointer: i32, message_length: i32) -> i32;
}

#[cfg(any(target_arch = "wasm32", test))]
type Callback =
    std::rc::Rc<std::cell::RefCell<Box<dyn FnMut(ChatMessage) -> SjmlResult<ChatMessageAction>>>>;
#[cfg(any(target_arch = "wasm32", test))]
struct CallbackEntry {
    callback: Callback,
    handle: Option<ChatMessageSubscriptionHandle>,
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
        F: FnMut(ChatMessage) -> SjmlResult<ChatMessageAction> + 'static,
    {
        if self.entries.len() >= MAX_CHAT_MESSAGE_SUBSCRIPTIONS {
            return Err(SjmlError::new(
                ErrorCode::RegistrationLimitReached,
                "client chat message subscription limit reached",
            ));
        }
        let id = self.next.ok_or_else(|| {
            SjmlError::new(
                ErrorCode::RegistrationLimitReached,
                "client chat callback identities exhausted",
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
}

#[cfg(any(target_arch = "wasm32", test))]
thread_local! {
    static CALLBACKS: std::cell::RefCell<Callbacks> = std::cell::RefCell::new(Callbacks::default());
    static DISPATCHING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Subscribes to local chat messages. Requires a declared and granted `client.chat.observe` capability.
/// Return `Hide` only to request local display filtering; the host ignores it unless
/// `client.chat.filter` is separately declared and granted.
pub fn subscribe<F>(callback: F) -> SjmlResult<ChatMessageSubscriptionHandle>
where
    F: FnMut(ChatMessage) -> SjmlResult<ChatMessageAction> + 'static,
{
    #[cfg(target_arch = "wasm32")]
    {
        let id = CALLBACKS.with(|state| state.borrow_mut().insert(callback))?;
        let output = allocate_host_input(HANDLE_BYTES);
        if output == 0 {
            CALLBACKS.with(|state| {
                state.borrow_mut().entries.remove(&id);
            });
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "could not allocate chat message subscription output",
            ));
        }
        // SAFETY: initialize the complete live output span before the host writes the stable handle.
        unsafe { core::ptr::write_bytes(output as *mut u8, 0, HANDLE_BYTES as usize) };
        // SAFETY: the output allocation remains live during this synchronous import.
        let status = unsafe { imported_subscribe(id as i64, output as i32) };
        let result = if status == 0 {
            // SAFETY: a successful host call initialized all 24 output bytes.
            from_bytes::<ChatMessageSubscriptionHandle>(unsafe {
                core::slice::from_raw_parts(output as *const u8, HANDLE_BYTES as usize)
            })
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected chat message subscription",
            ))
        };
        // SAFETY: release the exact allocation used for the host call.
        unsafe { deallocate_host_input(output, HANDLE_BYTES) };
        match result {
            Ok(handle) => {
                CALLBACKS.with(|state| {
                    if let Some(entry) = state.borrow_mut().entries.get_mut(&id) {
                        entry.handle = Some(handle);
                    }
                });
                Ok(handle)
            }
            Err(error) => {
                CALLBACKS.with(|state| {
                    state.borrow_mut().entries.remove(&id);
                });
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

/// Unsubscribes and removes the callback from the guest dispatch table.
pub fn unsubscribe(handle: ChatMessageSubscriptionHandle) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        if !CALLBACKS.with(|state| {
            state
                .borrow()
                .entries
                .values()
                .any(|entry| entry.handle == Some(handle))
        }) {
            return Err(stale());
        }
        let bytes = to_bytes(&handle)?;
        // SAFETY: the stable handle bytes remain live for the synchronous host call.
        let status = unsafe { imported_unsubscribe(bytes.as_ptr() as i32, bytes.len() as i32) };
        let result = host_status(status);
        if result.is_ok() || is_expired(status) {
            CALLBACKS.with(|state| {
                state
                    .borrow_mut()
                    .entries
                    .retain(|_, entry| entry.handle != Some(handle))
            });
        }
        result
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported())
    }
}

/// Sends one ordinary chat message through Minecraft's current local player connection.
///
/// Requires a declared and granted `client.chat.send` capability, a physical client, and a
/// connected local player. The host invokes Minecraft's `ClientPacketListener.sendChat` path,
/// including NeoForge's client-send hook, session signing, and acknowledgement tracking. It does
/// not expose command dispatch or packet construction. Messages must be nonblank, single-line,
/// free of control characters, not start with `/` after leading whitespace, and fit both Minecraft's
/// 256 UTF-16-unit input limit and the 1024-byte UTF-8 ABI limit. Each module runtime is limited to
/// one send per second.
pub fn send(message: &str) -> SjmlResult<()> {
    validate_outgoing_message(message)?;
    #[cfg(target_arch = "wasm32")]
    {
        let length = message.len() as u32;
        let pointer = allocate_host_input(length);
        if pointer == 0 {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "could not allocate client chat message input",
            ));
        }
        // SAFETY: the allocation is writable and exactly `length` bytes long.
        unsafe {
            core::ptr::copy_nonoverlapping(message.as_ptr(), pointer as *mut u8, length as usize)
        };
        // SAFETY: the initialized UTF-8 input remains live for the synchronous host call.
        let status = unsafe { imported_send(pointer as i32, length as i32) };
        // SAFETY: release the exact input allocation supplied to the host import.
        unsafe { deallocate_host_input(pointer, length) };
        host_status(status)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported())
    }
}

fn validate_outgoing_message(message: &str) -> SjmlResult<()> {
    if message.trim().is_empty() {
        return Err(invalid("chat text must not be blank"));
    }
    if message.len() > MAX_OUTGOING_CHAT_BYTES
        || message.encode_utf16().count() > MAX_OUTGOING_CHAT_UTF16_UNITS
    {
        return Err(invalid(
            "chat text exceeds the Minecraft UTF-16 or UTF-8 length limit",
        ));
    }
    if message
        .chars()
        .any(|character| character.is_control() || matches!(character, '\u{2028}' | '\u{2029}'))
    {
        return Err(invalid(
            "chat text must be single-line and contain no control characters",
        ));
    }
    if message.trim_start().starts_with('/') {
        return Err(invalid(
            "slash commands are not accepted by the chat-send API",
        ));
    }
    Ok(())
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug)]
struct Delivery {
    callback_id: u64,
    message: ChatMessage,
}

#[cfg(any(target_arch = "wasm32", test))]
impl StableDecode for Delivery {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        if decoder.read_u32()? != MESSAGE_SCHEMA_VERSION {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                "unsupported client chat message event schema",
            ));
        }
        let callback_id = decoder.read_u64()?;
        if callback_id == 0 || callback_id > i64::MAX as u64 {
            return Err(invalid("invalid chat callback identity"));
        }
        let source = match decoder.read_u8()? {
            0 => ChatMessageSource::PlayerChat,
            1 => ChatMessageSource::System,
            _ => return Err(invalid("unknown client chat message source")),
        };
        let flags = decoder.read_u8()?;
        if flags > 1 {
            return Err(invalid("unknown client chat message flags"));
        }
        let length = decoder.read_length(MAX_CHAT_MESSAGE_BYTES)?;
        let text = std::str::from_utf8(decoder.read_raw(length)?)
            .map_err(|_| invalid("client chat message is not valid UTF-8"))?
            .to_owned();
        Ok(Self {
            callback_id,
            message: ChatMessage {
                source,
                text,
                truncated: flags == 1,
            },
        })
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn deliver(delivery: Delivery) -> SjmlResult<bool> {
    if DISPATCHING.with(|state| state.replace(true)) {
        return Err(SjmlError::new(
            ErrorCode::ServiceBusy,
            "reentrant client chat message dispatch is forbidden",
        ));
    }
    let _guard = DispatchGuard;
    let callback = CALLBACKS.with(|state| {
        state
            .borrow()
            .entries
            .get(&delivery.callback_id)
            .filter(|entry| entry.handle.is_some())
            .map(|entry| entry.callback.clone())
            .ok_or_else(stale)
    })?;
    let mut callback = callback.try_borrow_mut().map_err(|_| {
        SjmlError::new(
            ErrorCode::ServiceBusy,
            "client chat message callback is already active",
        )
    })?;
    Ok(callback(delivery.message)? == ChatMessageAction::Hide)
}

#[cfg(any(target_arch = "wasm32", test))]
struct DispatchGuard;
#[cfg(any(target_arch = "wasm32", test))]
impl Drop for DispatchGuard {
    fn drop(&mut self) {
        DISPATCHING.with(|state| state.set(false));
    }
}

#[cfg(test)]
fn dispatch(bytes: &[u8]) -> SjmlResult<bool> {
    deliver(crate::from_bytes(bytes)?)
}

/// Receives a bounded host message snapshot and returns one stable bool byte.
///
/// # Safety
/// The host must provide a live initialized allocation returned by `sjml_v1_alloc` and keep it
/// readable until this synchronous call returns.
#[cfg(target_arch = "wasm32")]
#[export_name = "sjml_v1_client_chat_message_dispatch"]
pub unsafe extern "C" fn dispatch_export(pointer: u32, length: u32) -> u64 {
    // SAFETY: the caller guarantees the supplied guest allocation is live for this dispatch.
    unsafe { crate::abi::dispatch_payload(pointer, length, deliver) }
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
#[cfg(any(target_arch = "wasm32", test))]
fn stale() -> SjmlError {
    SjmlError::new(
        ErrorCode::StaleHandle,
        "chat message subscription is not active",
    )
}
fn invalid(message: &'static str) -> SjmlError {
    SjmlError::new(ErrorCode::InvalidArgument, message)
}
#[cfg(not(target_arch = "wasm32"))]
fn unsupported() -> SjmlError {
    SjmlError::new(
        ErrorCode::PermissionDenied,
        "client chat operations are available only to wasm guests",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};

    fn payload(source: u8, flags: u8, text: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&MESSAGE_SCHEMA_VERSION.to_le_bytes());
        bytes.extend_from_slice(&1u64.to_le_bytes());
        bytes.push(source);
        bytes.push(flags);
        bytes.extend_from_slice(&(text.len() as u32).to_le_bytes());
        bytes.extend_from_slice(text);
        bytes
    }

    #[test]
    fn decodes_bounded_player_and_system_snapshots() {
        let player = crate::from_bytes::<Delivery>(&payload(0, 0, "hello 🌍".as_bytes())).unwrap();
        assert_eq!(player.callback_id, 1);
        assert_eq!(player.message.source(), ChatMessageSource::PlayerChat);
        assert_eq!(player.message.text(), "hello 🌍");
        assert!(!player.message.was_truncated());

        let system = crate::from_bytes::<Delivery>(&payload(1, 1, b"notice")).unwrap();
        assert_eq!(system.message.source(), ChatMessageSource::System);
        assert!(system.message.was_truncated());
    }

    #[test]
    fn rejects_invalid_callback_source_flags_utf8_and_oversize_text() {
        for bytes in [
            payload(0, 2, b"x"),
            payload(2, 0, b"x"),
            payload(0, 0, &[0xff]),
        ] {
            assert!(crate::from_bytes::<Delivery>(&bytes).is_err());
        }
        assert_eq!(
            crate::from_bytes::<Delivery>(
                &payload(0, 0, b"x")
                    .into_iter()
                    .take(14)
                    .chain((MAX_CHAT_MESSAGE_BYTES as u32 + 1).to_le_bytes())
                    .collect::<Vec<_>>()
            )
            .unwrap_err()
            .code(),
            ErrorCode::PayloadTooLarge
        );
    }

    #[test]
    fn dispatch_invokes_only_the_matching_registered_callback() {
        let observed = Rc::new(RefCell::new(None));
        let target = observed.clone();
        let handle = ChatMessageSubscriptionHandle(crate::Handle::new(1, 2, 1, 1).unwrap());
        CALLBACKS.with(|state| {
            let mut state = state.borrow_mut();
            let id = state
                .insert(move |message| {
                    *target.borrow_mut() = Some(message.clone());
                    Ok(ChatMessageAction::Hide)
                })
                .unwrap();
            state.entries.get_mut(&id).unwrap().handle = Some(handle);
        });
        assert!(dispatch(&payload(1, 0, b"system")).unwrap());
        let received = observed.borrow().clone().unwrap();
        assert_eq!(received.source(), ChatMessageSource::System);
        assert_eq!(received.text(), "system");
        CALLBACKS.with(|state| {
            state.borrow_mut().entries.clear();
        });
    }

    #[test]
    fn outgoing_chat_validation_rejects_blank_controls_lines_commands_and_overflow() {
        assert!(validate_outgoing_message("hello 🌍").is_ok());
        assert!(validate_outgoing_message(&"x".repeat(MAX_OUTGOING_CHAT_UTF16_UNITS)).is_ok());
        let overlong = "x".repeat(MAX_OUTGOING_CHAT_UTF16_UNITS + 1);
        for message in [
            "",
            " \t ",
            "one\ntwo",
            "one\rtwo",
            "one\u{2028}two",
            "/say hello",
            "   /say hello",
            overlong.as_str(),
        ] {
            assert_eq!(
                validate_outgoing_message(message).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
        }
    }
}
