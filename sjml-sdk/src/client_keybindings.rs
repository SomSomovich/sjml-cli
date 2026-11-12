//! Declarative client key mappings and bounded key-down callbacks.

use crate::abi::EventSubscriptionHandle;
use crate::{ErrorCode, SjmlError, SjmlResult};

/// Maximum keybinding callbacks owned by one module.
pub const MAX_CLIENT_KEYBINDING_SUBSCRIPTIONS: usize = 128;
const MAX_PATH_BYTES: usize = 48;
#[cfg(target_arch = "wasm32")]
const HANDLE_BYTES: u32 = 24;

/// Opaque module-owned handle for a keybinding callback.
pub type KeybindingSubscriptionHandle = EventSubscriptionHandle;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Key {
    /// GLFW/NeoForge keyboard symbol. `-1` means unbound.
    Keyboard(i32),
    /// GLFW mouse button number, from 0 through 31.
    Mouse(u8),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Modifier {
    #[default]
    None,
    Shift,
    Control,
    Alt,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Context {
    /// Active while the player is in game and no GUI screen is open.
    #[default]
    InGame,
    /// Active while a GUI screen is open.
    Gui,
    /// Active in both game and GUI contexts.
    Universal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KeybindingDefinition {
    pub key: Key,
    pub modifier: Modifier,
    pub context: Context,
}

impl KeybindingDefinition {
    pub const fn new(key: Key) -> Self {
        Self {
            key,
            modifier: Modifier::None,
            context: Context::InGame,
        }
    }

    pub const fn modifier(mut self, modifier: Modifier) -> Self {
        self.modifier = modifier;
        self
    }

    pub const fn context(mut self, context: Context) -> Self {
        self.context = context;
        self
    }

    fn wire(self) -> SjmlResult<(i32, i32, i32, i32)> {
        let (kind, code) = match self.key {
            Key::Keyboard(code) if (-1..=65_535).contains(&code) => (0, code),
            Key::Keyboard(_) => {
                return Err(invalid("keyboard key code must be between -1 and 65535"));
            }
            Key::Mouse(button) if button <= 31 => (1, i32::from(button)),
            Key::Mouse(_) => return Err(invalid("mouse button must be between 0 and 31")),
        };
        let modifier = match self.modifier {
            Modifier::None => 0,
            Modifier::Shift => 1,
            Modifier::Control => 2,
            Modifier::Alt => 3,
        };
        let context = match self.context {
            Context::InGame => 0,
            Context::Gui => 1,
            Context::Universal => 2,
        };
        Ok((kind, code, modifier, context))
    }
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_client_keybinding_v1")]
extern "C" {
    #[link_name = "register"]
    fn imported_register(
        path_pointer: i32,
        path_length: i32,
        key_type: i32,
        code: i32,
        modifier: i32,
        context: i32,
    ) -> i32;
    #[link_name = "subscribe"]
    fn imported_subscribe(
        callback_id: i64,
        path_pointer: i32,
        path_length: i32,
        handle_output_pointer: i32,
    ) -> i32;
    #[link_name = "unsubscribe"]
    fn imported_unsubscribe(handle_pointer: i32, handle_length: i32) -> i32;
}

#[cfg(target_arch = "wasm32")]
type Callback = std::rc::Rc<std::cell::RefCell<Box<dyn FnMut() -> SjmlResult<()>>>>;
#[cfg(target_arch = "wasm32")]
struct Entry {
    callback: Callback,
    handle: Option<KeybindingSubscriptionHandle>,
}
#[cfg(target_arch = "wasm32")]
#[derive(Default)]
struct Callbacks {
    next: Option<u64>,
    entries: std::collections::BTreeMap<u64, Entry>,
}

/// Stages a key mapping from `#[mod_register]`. Call `subscribe` during `mod_init` to attach behavior.
pub fn register(path: &str, definition: KeybindingDefinition) -> SjmlResult<()> {
    validate_path(path)?;
    let (key_type, code, modifier, context) = definition.wire()?;
    #[cfg(target_arch = "wasm32")]
    {
        let status = unsafe {
            imported_register(
                path.as_ptr() as i32,
                path.len() as i32,
                key_type,
                code,
                modifier,
                context,
            )
        };
        if status != 0 {
            return Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected keybinding declaration",
            ));
        }
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (key_type, code, modifier, context);
        Err(unsupported())
    }
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static CALLBACKS: std::cell::RefCell<Callbacks> = std::cell::RefCell::new(Callbacks::default());
    static DISPATCHING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Attaches a key-down callback to a mapping declared with [`register`].
pub fn subscribe<F>(path: &str, callback: F) -> SjmlResult<KeybindingSubscriptionHandle>
where
    F: FnMut() -> SjmlResult<()> + 'static,
{
    validate_path(path)?;
    #[cfg(target_arch = "wasm32")]
    {
        let callback_id = CALLBACKS.with(|items| {
            let mut items = items.borrow_mut();
            if items.entries.len() >= MAX_CLIENT_KEYBINDING_SUBSCRIPTIONS {
                return Err(SjmlError::new(
                    ErrorCode::RegistrationLimitReached,
                    "keybinding callback limit reached",
                ));
            }
            let id = items.next.unwrap_or(1);
            items.next = id.checked_add(1);
            items.entries.insert(
                id,
                Entry {
                    callback: std::rc::Rc::new(std::cell::RefCell::new(Box::new(callback))),
                    handle: None,
                },
            );
            Ok(id)
        })?;
        let output = crate::abi::allocate_host_input(HANDLE_BYTES);
        if output == 0 {
            CALLBACKS.with(|items| {
                items.borrow_mut().entries.remove(&callback_id);
            });
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "could not allocate keybinding handle output",
            ));
        }
        unsafe { core::ptr::write_bytes(output as *mut u8, 0, HANDLE_BYTES as usize) };
        let status = unsafe {
            imported_subscribe(
                callback_id as i64,
                path.as_ptr() as i32,
                path.len() as i32,
                output as i32,
            )
        };
        let result = if status == 0 {
            crate::from_bytes::<KeybindingSubscriptionHandle>(unsafe {
                core::slice::from_raw_parts(output as *const u8, HANDLE_BYTES as usize)
            })
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected keybinding subscription",
            ))
        };
        unsafe { crate::abi::deallocate_host_input(output, HANDLE_BYTES) };
        match result {
            Ok(handle) => {
                CALLBACKS.with(|items| {
                    if let Some(entry) = items.borrow_mut().entries.get_mut(&callback_id) {
                        entry.handle = Some(handle);
                    }
                });
                Ok(handle)
            }
            Err(error) => {
                CALLBACKS.with(|items| {
                    items.borrow_mut().entries.remove(&callback_id);
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

pub fn unsubscribe(handle: KeybindingSubscriptionHandle) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let callback_id = CALLBACKS
            .with(|items| {
                items
                    .borrow()
                    .entries
                    .iter()
                    .find_map(|(&id, entry)| (entry.handle == Some(handle)).then_some(id))
            })
            .ok_or_else(|| {
                SjmlError::new(
                    ErrorCode::StaleHandle,
                    "keybinding subscription is not active",
                )
            })?;
        let bytes = crate::to_bytes(&handle)?;
        let status = unsafe { imported_unsubscribe(bytes.as_ptr() as i32, bytes.len() as i32) };
        if status != 0 {
            return Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected keybinding unsubscription",
            ));
        }
        CALLBACKS.with(|items| {
            items.borrow_mut().entries.remove(&callback_id);
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
struct Delivery {
    callback_id: u64,
}
#[cfg(target_arch = "wasm32")]
impl crate::StableDecode for Delivery {
    fn decode(decoder: &mut crate::Decoder<'_>) -> SjmlResult<Self> {
        if decoder.read_u32()? != 1 {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                "unsupported keybinding schema",
            ));
        }
        let callback_id = decoder.read_u64()?;
        if callback_id == 0 {
            return Err(invalid("keybinding callback ID is zero"));
        }
        Ok(Self { callback_id })
    }
}
#[cfg(target_arch = "wasm32")]
impl crate::StableEncode for Delivery {
    fn encode(&self, encoder: &mut crate::Encoder) -> SjmlResult<()> {
        encoder.write_u32(1)?;
        encoder.write_u64(self.callback_id)
    }
}

/// Bounded host callback entry point for key-down notifications.
#[cfg(target_arch = "wasm32")]
#[export_name = "sjml_v1_client_keybinding_dispatch"]
pub unsafe extern "C" fn dispatch(pointer: u32, length: u32) -> u64 {
    unsafe {
        crate::abi::dispatch_payload(pointer, length, |delivery: Delivery| {
            if DISPATCHING.with(|active| active.replace(true)) {
                return Err(SjmlError::new(
                    ErrorCode::ServiceBusy,
                    "reentrant keybinding callback is forbidden",
                ));
            }
            let result = (|| {
                let callback = CALLBACKS.with(|items| {
                    items
                        .borrow()
                        .entries
                        .get(&delivery.callback_id)
                        .map(|entry| entry.callback.clone())
                        .ok_or_else(|| {
                            SjmlError::new(
                                ErrorCode::StaleHandle,
                                "keybinding callback is not subscribed",
                            )
                        })
                })?;
                let mut callback = callback.try_borrow_mut().map_err(|_| {
                    SjmlError::new(
                        ErrorCode::ServiceBusy,
                        "keybinding callback is already active",
                    )
                })?;
                callback()
            })();
            DISPATCHING.with(|active| active.set(false));
            result
        })
    }
}

fn validate_path(path: &str) -> SjmlResult<()> {
    if path.is_empty()
        || path.len() > MAX_PATH_BYTES
        || !path.bytes().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'_' | b'.' | b'-')
        })
        || !path.as_bytes()[0].is_ascii_lowercase() && !path.as_bytes()[0].is_ascii_digit()
        || path.contains("..")
        || path.ends_with('.')
    {
        return Err(invalid(
            "keybinding path must be 1-48 lowercase ASCII resource characters",
        ));
    }
    Ok(())
}

fn invalid(message: &'static str) -> SjmlError {
    SjmlError::new(ErrorCode::InvalidArgument, message)
}
#[cfg(not(target_arch = "wasm32"))]
fn unsupported() -> SjmlError {
    SjmlError::new(
        ErrorCode::PermissionDenied,
        "keybindings are available only in a wasm32 client guest",
    )
}

#[cfg(test)]
mod tests {
    use super::{Context, Key, KeybindingDefinition, Modifier};

    #[test]
    fn validates_key_codes_and_encodes_configuration_values() {
        assert_eq!(
            KeybindingDefinition::new(Key::Keyboard(65))
                .modifier(Modifier::Control)
                .context(Context::Universal)
                .wire()
                .unwrap(),
            (0, 65, 2, 2)
        );
        assert_eq!(
            KeybindingDefinition::new(Key::Mouse(31)).wire().unwrap(),
            (1, 31, 0, 0)
        );
        assert!(KeybindingDefinition::new(Key::Keyboard(65_536))
            .wire()
            .is_err());
        assert!(KeybindingDefinition::new(Key::Mouse(32)).wire().is_err());
    }

    #[test]
    fn path_bounds_and_character_set_are_enforced() {
        assert!(super::validate_path("open_menu").is_ok());
        assert!(super::validate_path("A_bad").is_err());
        assert!(super::validate_path("a..b").is_err());
        assert!(super::validate_path(&"a".repeat(49)).is_err());
    }
}
