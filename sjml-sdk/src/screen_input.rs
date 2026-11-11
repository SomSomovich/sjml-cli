//! Bounded active-screen mouse, keyboard, and character callbacks for client guest modules.

use crate::abi::EventSubscriptionHandle;
#[cfg(target_arch = "wasm32")]
use crate::abi::{allocate_host_input, deallocate_host_input};
#[cfg(target_arch = "wasm32")]
use crate::StableDecode;
#[cfg(target_arch = "wasm32")]
use crate::{from_bytes, to_bytes};
use crate::{ErrorCode, SjmlError, SjmlResult};

/// Maximum number of active-screen input callbacks owned by one guest module.
pub const MAX_SCREEN_INPUT_SUBSCRIPTIONS: usize = 256;
/// Maximum absolute scaled GUI coordinate accepted in an input event.
pub const MAX_SCREEN_INPUT_COORDINATE: f32 = 1_048_576.0;
#[cfg(target_arch = "wasm32")]
const SCREEN_INPUT_SCHEMA_VERSION: u32 = 1;
#[cfg(target_arch = "wasm32")]
const HANDLE_BYTES: u32 = 24;

/// Module-owned active-screen input subscription handle.
pub type ScreenInputSubscriptionHandle = EventSubscriptionHandle;

/// A click or key press from the active screen, captured on the client main thread.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScreenInputEvent {
    /// Mouse click coordinates are scaled GUI units. Buttons range from 0 through 31;
    /// modifier bits are Shift, Control, Alt, Super, Caps Lock, and Num Lock.
    MouseButtonPressed {
        mouse_x: f32,
        mouse_y: f32,
        button: u8,
        modifiers: u8,
    },
    /// Key code -1 represents an unknown GLFW key with a valid scan code.
    KeyPressed {
        key_code: i32,
        scan_code: u16,
        modifiers: u8,
        repeat: bool,
    },
    /// One UTF-16 code unit from the active screen's character callback. A supplementary Unicode
    /// scalar is delivered as a high-surrogate and low-surrogate pair; use the `TextField` widget
    /// to combine and edit those units safely.
    CharacterTyped { code_unit: u16, modifiers: u8 },
}

impl ScreenInputEvent {
    #[cfg(any(target_arch = "wasm32", test))]
    fn validate(self) -> SjmlResult<Self> {
        match self {
            Self::MouseButtonPressed {
                mouse_x,
                mouse_y,
                button,
                modifiers,
            } => {
                validate_coordinate(mouse_x)?;
                validate_coordinate(mouse_y)?;
                if button > 31 {
                    return Err(invalid("mouse button is outside the supported range"));
                }
                validate_modifiers(modifiers)?;
            }
            Self::KeyPressed {
                key_code,
                modifiers,
                ..
            } => {
                if !(-1..=65_535).contains(&key_code) {
                    return Err(invalid("key code is outside the supported range"));
                }
                validate_modifiers(modifiers)?;
            }
            Self::CharacterTyped { modifiers, .. } => validate_modifiers(modifiers)?,
        }
        Ok(self)
    }
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_screen_input_v1")]
extern "C" {
    #[link_name = "subscribe"]
    fn imported_subscribe(callback_id: i64, handle_output_pointer: i32) -> i32;
    #[link_name = "unsubscribe"]
    fn imported_unsubscribe(handle_pointer: i32, handle_length: i32) -> i32;
}

#[cfg(target_arch = "wasm32")]
type Callback =
    std::rc::Rc<std::cell::RefCell<Box<dyn FnMut(ScreenInputEvent) -> SjmlResult<bool>>>>;

#[cfg(target_arch = "wasm32")]
struct CallbackEntry {
    callback: Callback,
    handle: Option<ScreenInputSubscriptionHandle>,
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
        F: FnMut(ScreenInputEvent) -> SjmlResult<bool> + 'static,
    {
        if self.entries.len() >= MAX_SCREEN_INPUT_SUBSCRIPTIONS {
            return Err(SjmlError::new(
                ErrorCode::RegistrationLimitReached,
                "screen input subscription limit reached",
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
                    "screen input callback is not subscribed",
                )
            })
    }
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static CALLBACKS: std::cell::RefCell<Callbacks> = std::cell::RefCell::new(Callbacks::default());
    static DISPATCHING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Subscribes a typed callback. Return `true` to consume the mouse, key, or character event.
pub fn subscribe<F>(callback: F) -> SjmlResult<ScreenInputSubscriptionHandle>
where
    F: FnMut(ScreenInputEvent) -> SjmlResult<bool> + 'static,
{
    #[cfg(target_arch = "wasm32")]
    {
        let callback_id = CALLBACKS.with(|registry| registry.borrow_mut().insert(callback))?;
        let output = allocate_host_input(HANDLE_BYTES);
        if output == 0 {
            CALLBACKS.with(|registry| registry.borrow_mut().entries.remove(&callback_id));
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "could not allocate screen input subscription output",
            ));
        }
        // SAFETY: initialize the complete live output allocation before the host writes the handle.
        unsafe { core::ptr::write_bytes(output as *mut u8, 0, HANDLE_BYTES as usize) };
        // SAFETY: the exact output allocation remains live during the synchronous host import.
        let status = unsafe { imported_subscribe(callback_id as i64, output as i32) };
        let result = if status == 0 {
            // SAFETY: a successful host call initialized the full handle allocation.
            from_bytes::<ScreenInputSubscriptionHandle>(unsafe {
                core::slice::from_raw_parts(output as *const u8, HANDLE_BYTES as usize)
            })
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected screen input subscription",
            ))
        };
        // SAFETY: release the same allocation returned above after the synchronous call.
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

/// Removes an active subscription from both the host and guest callback registry.
pub fn unsubscribe(handle: ScreenInputSubscriptionHandle) -> SjmlResult<()> {
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
                    "screen input subscription is not active in this module",
                )
            })?;
        let bytes = to_bytes(&handle)?;
        // SAFETY: the stable handle bytes live for the complete synchronous host call.
        let status = unsafe { imported_unsubscribe(bytes.as_ptr() as i32, bytes.len() as i32) };
        if status != 0 {
            return Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected screen input unsubscription",
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
struct ScreenInputDelivery {
    callback_id: u64,
    event: ScreenInputEvent,
}

#[cfg(target_arch = "wasm32")]
impl StableDecode for ScreenInputDelivery {
    fn decode(decoder: &mut crate::Decoder<'_>) -> SjmlResult<Self> {
        if decoder.read_u32()? != SCREEN_INPUT_SCHEMA_VERSION {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                "unsupported screen input dispatch schema",
            ));
        }
        let callback_id = decoder.read_u64()?;
        if callback_id == 0 {
            return Err(invalid("screen input callback identity is zero"));
        }
        let event = match decoder.read_u8()? {
            0 => ScreenInputEvent::MouseButtonPressed {
                mouse_x: decoder.read_f32()?,
                mouse_y: decoder.read_f32()?,
                button: decoder.read_u8()?,
                modifiers: decoder.read_u8()?,
            },
            1 => ScreenInputEvent::KeyPressed {
                key_code: decoder.read_i32()?,
                scan_code: decoder.read_u16()?,
                modifiers: decoder.read_u8()?,
                repeat: bool::decode(decoder)?,
            },
            2 => ScreenInputEvent::CharacterTyped {
                code_unit: decoder.read_u16()?,
                modifiers: decoder.read_u8()?,
            },
            _ => return Err(invalid("unknown screen input event type")),
        }
        .validate()?;
        Ok(Self { callback_id, event })
    }
}

/// Dispatches one host screen-input callback and returns its consume decision as a stable bool.
///
/// # Safety
/// The host must pass a live initialized input allocation from `sjml_v1_alloc` and keep it alive
/// until return, then release input and response with the paired ABI dealloc exports.
#[cfg(target_arch = "wasm32")]
#[export_name = "sjml_v1_screen_input_dispatch"]
pub unsafe extern "C" fn dispatch(pointer: u32, length: u32) -> u64 {
    // SAFETY: the host guarantees the callback input allocation's initialized live span.
    unsafe {
        crate::abi::dispatch_payload(pointer, length, |delivery: ScreenInputDelivery| {
            if DISPATCHING.with(|active| active.replace(true)) {
                return Err(SjmlError::new(
                    ErrorCode::ServiceBusy,
                    "reentrant screen input dispatch is forbidden",
                ));
            }
            let result = (|| {
                let callback =
                    CALLBACKS.with(|registry| registry.borrow().get(delivery.callback_id))?;
                let mut callback = callback.try_borrow_mut().map_err(|_| {
                    SjmlError::new(
                        ErrorCode::ServiceBusy,
                        "screen input callback is already active",
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
fn validate_coordinate(value: f32) -> SjmlResult<()> {
    if !value.is_finite() || value.abs() > MAX_SCREEN_INPUT_COORDINATE {
        return Err(invalid(
            "screen input coordinates are outside the supported finite range",
        ));
    }
    Ok(())
}

#[cfg(any(target_arch = "wasm32", test))]
fn validate_modifiers(value: u8) -> SjmlResult<()> {
    if value & !0x3f != 0 {
        return Err(invalid("screen input modifiers contain unsupported bits"));
    }
    Ok(())
}

#[cfg(any(target_arch = "wasm32", test))]
fn invalid(message: &'static str) -> SjmlError {
    SjmlError::new(ErrorCode::InvalidArgument, message)
}

#[cfg(not(target_arch = "wasm32"))]
fn unsupported() -> SjmlError {
    SjmlError::new(
        ErrorCode::PermissionDenied,
        "screen input subscriptions are available only in a wasm32 client guest",
    )
}

#[cfg(test)]
mod tests {
    use super::{ScreenInputEvent, MAX_SCREEN_INPUT_COORDINATE};

    #[test]
    fn validates_bounded_click_and_key_events() {
        let click = ScreenInputEvent::MouseButtonPressed {
            mouse_x: MAX_SCREEN_INPUT_COORDINATE,
            mouse_y: -MAX_SCREEN_INPUT_COORDINATE,
            button: 7,
            modifiers: 0x03,
        };
        assert_eq!(click.validate().unwrap(), click);
        let key = ScreenInputEvent::KeyPressed {
            key_code: -1,
            scan_code: 65_535,
            modifiers: 0x20,
            repeat: true,
        };
        assert_eq!(key.validate().unwrap(), key);
    }

    #[test]
    fn rejects_non_finite_coordinates_and_unknown_modifiers() {
        assert!(ScreenInputEvent::MouseButtonPressed {
            mouse_x: f32::NAN,
            mouse_y: 0.0,
            button: 0,
            modifiers: 0,
        }
        .validate()
        .is_err());
        assert!(ScreenInputEvent::KeyPressed {
            key_code: 65_536,
            scan_code: 1,
            modifiers: 0,
            repeat: false,
        }
        .validate()
        .is_err());
        assert!(ScreenInputEvent::KeyPressed {
            key_code: 1,
            scan_code: 1,
            modifiers: 0x40,
            repeat: false,
        }
        .validate()
        .is_err());
        let typed = ScreenInputEvent::CharacterTyped {
            code_unit: 0xd83d,
            modifiers: 0x01,
        };
        assert_eq!(typed.validate().unwrap(), typed);
        assert!(ScreenInputEvent::CharacterTyped {
            code_unit: b'a' as u16,
            modifiers: 0x40,
        }
        .validate()
        .is_err());
    }
}
