//! Bounded global keyboard and mouse-button callbacks for client guests.

use crate::abi::EventSubscriptionHandle;
#[cfg(target_arch = "wasm32")]
use crate::abi::{allocate_host_input, deallocate_host_input};
#[cfg(target_arch = "wasm32")]
use crate::StableDecode;
#[cfg(target_arch = "wasm32")]
use crate::{from_bytes, to_bytes};
use crate::{ErrorCode, SjmlError, SjmlResult};

/// Maximum number of global client input callbacks owned by one module.
pub const MAX_CLIENT_INPUT_SUBSCRIPTIONS: usize = 256;
#[cfg(target_arch = "wasm32")]
const CLIENT_INPUT_SCHEMA_VERSION: u32 = 1;
#[cfg(target_arch = "wasm32")]
const HANDLE_BYTES: u32 = 24;

/// Module-owned global client-input subscription handle.
pub type ClientInputSubscriptionHandle = EventSubscriptionHandle;

/// A keyboard or mouse-button notification received on the client main thread.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientInputEvent {
    /// GLFW key code -1 represents an unknown key with a valid scan code.
    Key {
        key_code: i32,
        scan_code: u16,
        action: ClientInputAction,
        modifiers: u8,
    },
    /// Mouse button indexes are in 0..=31. GLFW has no mouse repeat action.
    MouseButton {
        button: u8,
        action: ClientInputAction,
        modifiers: u8,
    },
}

/// Input action values reported by GLFW/NeoForge.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientInputAction {
    Press,
    Release,
    Repeat,
}

impl ClientInputEvent {
    #[cfg(any(target_arch = "wasm32", test))]
    fn validate(self) -> SjmlResult<Self> {
        match self {
            Self::Key {
                key_code,
                modifiers,
                ..
            } => {
                if !(-1..=65_535).contains(&key_code) {
                    return Err(invalid(
                        "client input key code is outside its supported range",
                    ));
                }
                validate_modifiers(modifiers)?;
            }
            Self::MouseButton {
                button,
                action,
                modifiers,
            } => {
                if button > 31 || action == ClientInputAction::Repeat {
                    return Err(invalid("client input mouse button or action is invalid"));
                }
                validate_modifiers(modifiers)?;
            }
        }
        Ok(self)
    }

    #[cfg(target_arch = "wasm32")]
    fn kind_wire(self) -> u8 {
        match self {
            Self::Key { .. } => 0,
            Self::MouseButton { .. } => 1,
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn fields(self) -> (ClientInputAction, i32, u16, u8) {
        match self {
            Self::Key {
                key_code,
                scan_code,
                action,
                modifiers,
            } => (action, key_code, scan_code, modifiers),
            Self::MouseButton {
                button,
                action,
                modifiers,
            } => (action, i32::from(button), 0, modifiers),
        }
    }
}

impl ClientInputAction {
    #[cfg(target_arch = "wasm32")]
    fn wire_value(self) -> u8 {
        match self {
            Self::Press => 0,
            Self::Release => 1,
            Self::Repeat => 2,
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn from_wire(value: u8) -> SjmlResult<Self> {
        match value {
            0 => Ok(Self::Press),
            1 => Ok(Self::Release),
            2 => Ok(Self::Repeat),
            _ => Err(invalid("unknown client input action")),
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_client_input_v1")]
extern "C" {
    #[link_name = "subscribe"]
    fn imported_subscribe(callback_id: i64, handle_output_pointer: i32) -> i32;
    #[link_name = "unsubscribe"]
    fn imported_unsubscribe(handle_pointer: i32, handle_length: i32) -> i32;
}

#[cfg(target_arch = "wasm32")]
type Callback =
    std::rc::Rc<std::cell::RefCell<Box<dyn FnMut(ClientInputEvent) -> SjmlResult<bool>>>>;

#[cfg(target_arch = "wasm32")]
struct CallbackEntry {
    callback: Callback,
    handle: Option<ClientInputSubscriptionHandle>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Default)]
struct Callbacks {
    next: Option<u64>,
    entries: std::collections::BTreeMap<u64, CallbackEntry>,
}

#[cfg(target_arch = "wasm32")]
impl Callbacks {
    fn insert<F>(&mut self, callback: F) -> SjmlResult<u64>
    where
        F: FnMut(ClientInputEvent) -> SjmlResult<bool> + 'static,
    {
        if self.entries.len() >= MAX_CLIENT_INPUT_SUBSCRIPTIONS {
            return Err(SjmlError::new(
                ErrorCode::RegistrationLimitReached,
                "client input subscription limit reached",
            ));
        }
        let id = self.next.unwrap_or(1);
        self.next = id.checked_add(1);
        self.entries.insert(
            id,
            CallbackEntry {
                callback: std::rc::Rc::new(std::cell::RefCell::new(Box::new(callback))),
                handle: None,
            },
        );
        Ok(id)
    }

    fn get(&self, id: u64) -> SjmlResult<Callback> {
        self.entries
            .get(&id)
            .map(|entry| entry.callback.clone())
            .ok_or_else(|| {
                SjmlError::new(
                    ErrorCode::StaleHandle,
                    "client input callback is not subscribed",
                )
            })
    }
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static CALLBACKS: std::cell::RefCell<Callbacks> = std::cell::RefCell::new(Callbacks::default());
    static DISPATCHING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Subscribes a typed callback. Return `true` to consume a mouse-button press. NeoForge global key
/// notifications and all releases are observable but are not cancellable.
pub fn subscribe<F>(callback: F) -> SjmlResult<ClientInputSubscriptionHandle>
where
    F: FnMut(ClientInputEvent) -> SjmlResult<bool> + 'static,
{
    #[cfg(target_arch = "wasm32")]
    {
        let callback_id = CALLBACKS.with(|registry| registry.borrow_mut().insert(callback))?;
        let output = allocate_host_input(HANDLE_BYTES);
        if output == 0 {
            CALLBACKS.with(|registry| registry.borrow_mut().entries.remove(&callback_id));
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "could not allocate client input subscription output",
            ));
        }
        // SAFETY: initialize the complete live output allocation before the host writes the handle.
        unsafe { core::ptr::write_bytes(output as *mut u8, 0, HANDLE_BYTES as usize) };
        // SAFETY: the exact output allocation remains live during the synchronous host import.
        let status = unsafe { imported_subscribe(callback_id as i64, output as i32) };
        let result = if status == 0 {
            // SAFETY: a successful call initialized the complete handle allocation.
            from_bytes::<ClientInputSubscriptionHandle>(unsafe {
                core::slice::from_raw_parts(output as *const u8, HANDLE_BYTES as usize)
            })
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected client input subscription",
            ))
        };
        // SAFETY: release the exact allocation passed to the host import.
        unsafe { deallocate_host_input(output, HANDLE_BYTES) };
        match result {
            Ok(handle) => {
                CALLBACKS.with(|registry| {
                    if let Some(entry) = registry.borrow_mut().entries.get_mut(&callback_id) {
                        entry.handle = Some(handle);
                    }
                });
                Ok(handle)
            }
            Err(error) => {
                CALLBACKS.with(|registry| registry.borrow_mut().entries.remove(&callback_id));
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

/// Removes an active subscription from the host and guest callback registry.
pub fn unsubscribe(handle: ClientInputSubscriptionHandle) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let callback_id = CALLBACKS
            .with(|registry| {
                registry
                    .borrow()
                    .entries
                    .iter()
                    .find_map(|(&id, entry)| (entry.handle == Some(handle)).then_some(id))
            })
            .ok_or_else(|| {
                SjmlError::new(
                    ErrorCode::StaleHandle,
                    "client input subscription is not active in this module",
                )
            })?;
        let bytes = to_bytes(&handle)?;
        // SAFETY: the stable handle bytes live for the complete synchronous host call.
        let status = unsafe { imported_unsubscribe(bytes.as_ptr() as i32, bytes.len() as i32) };
        if status != 0 {
            return Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected client input unsubscription",
            ));
        }
        CALLBACKS.with(|registry| {
            registry.borrow_mut().entries.remove(&callback_id);
        });
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported())
    }
}

#[cfg(target_arch = "wasm32")]
struct ClientInputDelivery {
    callback_id: u64,
    event: ClientInputEvent,
}

#[cfg(target_arch = "wasm32")]
impl StableDecode for ClientInputDelivery {
    fn decode(decoder: &mut crate::Decoder<'_>) -> SjmlResult<Self> {
        if decoder.read_u32()? != CLIENT_INPUT_SCHEMA_VERSION {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                "unsupported client input dispatch schema",
            ));
        }
        let callback_id = decoder.read_u64()?;
        if callback_id == 0 {
            return Err(invalid("client input callback identity is zero"));
        }
        let kind = decoder.read_u8()?;
        let action = ClientInputAction::from_wire(decoder.read_u8()?)?;
        let code = decoder.read_i32()?;
        let scan_code = decoder.read_u16()?;
        let modifiers = decoder.read_u8()?;
        let event = match kind {
            0 => ClientInputEvent::Key {
                key_code: code,
                scan_code,
                action,
                modifiers,
            },
            1 => ClientInputEvent::MouseButton {
                button: u8::try_from(code)
                    .map_err(|_| invalid("mouse button is outside the supported range"))?,
                action,
                modifiers,
            },
            _ => return Err(invalid("unknown client input event kind")),
        }
        .validate()?;
        Ok(Self { callback_id, event })
    }
}

#[cfg(target_arch = "wasm32")]
impl crate::StableEncode for ClientInputDelivery {
    fn encode(&self, encoder: &mut crate::Encoder) -> SjmlResult<()> {
        let (action, code, scan_code, modifiers) = self.event.fields();
        encoder.write_u32(CLIENT_INPUT_SCHEMA_VERSION)?;
        encoder.write_u64(self.callback_id)?;
        encoder.write_u8(self.event.kind_wire())?;
        encoder.write_u8(action.wire_value())?;
        encoder.write_i32(code)?;
        encoder.write_u16(scan_code)?;
        encoder.write_u8(modifiers)
    }
}

/// Dispatches one host client-input callback and returns its consume decision as a stable bool.
///
/// # Safety
/// The host must pass a live initialized guest allocation and keep it alive until return, then
/// release the input and response with the paired ABI dealloc exports.
#[cfg(target_arch = "wasm32")]
#[export_name = "sjml_v1_client_input_dispatch"]
pub unsafe extern "C" fn dispatch(pointer: u32, length: u32) -> u64 {
    // SAFETY: the host guarantees the input allocation's initialized live span.
    unsafe {
        crate::abi::dispatch_payload(pointer, length, |delivery: ClientInputDelivery| {
            if DISPATCHING.with(|active| active.replace(true)) {
                return Err(SjmlError::new(
                    ErrorCode::ServiceBusy,
                    "reentrant client input dispatch is forbidden",
                ));
            }
            let result = (|| {
                let callback =
                    CALLBACKS.with(|registry| registry.borrow().get(delivery.callback_id))?;
                let mut callback = callback.try_borrow_mut().map_err(|_| {
                    SjmlError::new(
                        ErrorCode::ServiceBusy,
                        "client input callback is already active",
                    )
                })?;
                callback(delivery.event)
            })();
            DISPATCHING.with(|active| active.set(false));
            result
        })
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn invalid(message: &'static str) -> SjmlError {
    SjmlError::new(ErrorCode::InvalidArgument, message)
}

#[cfg(any(target_arch = "wasm32", test))]
fn validate_modifiers(modifiers: u8) -> SjmlResult<()> {
    if modifiers & !0x3f != 0 {
        return Err(invalid("client input modifiers contain unsupported bits"));
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn unsupported() -> SjmlError {
    SjmlError::new(
        ErrorCode::PermissionDenied,
        "client input subscriptions are available only in a wasm32 client guest",
    )
}

#[cfg(test)]
mod tests {
    use super::{ClientInputAction, ClientInputEvent};

    #[test]
    fn validates_key_and_mouse_actions_with_bounded_codes_and_modifiers() {
        let key = ClientInputEvent::Key {
            key_code: -1,
            scan_code: u16::MAX,
            action: ClientInputAction::Repeat,
            modifiers: 0x3f,
        };
        assert_eq!(key.validate().unwrap(), key);
        let mouse = ClientInputEvent::MouseButton {
            button: 31,
            action: ClientInputAction::Release,
            modifiers: 0,
        };
        assert_eq!(mouse.validate().unwrap(), mouse);
    }

    #[test]
    fn rejects_invalid_code_action_and_modifier_values() {
        assert!(ClientInputEvent::Key {
            key_code: 65_536,
            scan_code: 0,
            action: ClientInputAction::Press,
            modifiers: 0,
        }
        .validate()
        .is_err());
        assert!(ClientInputEvent::MouseButton {
            button: 32,
            action: ClientInputAction::Press,
            modifiers: 0,
        }
        .validate()
        .is_err());
        assert!(ClientInputEvent::MouseButton {
            button: 0,
            action: ClientInputAction::Repeat,
            modifiers: 0,
        }
        .validate()
        .is_err());
        assert!(ClientInputEvent::Key {
            key_code: 1,
            scan_code: 1,
            action: ClientInputAction::Press,
            modifiers: 0x40,
        }
        .validate()
        .is_err());
    }
}
