//! Bounded active-screen mouse, wheel, keyboard, and character callbacks for client guest modules.

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
/// Maximum absolute axis delta accepted for active-screen mouse-wheel input.
pub const MAX_SCREEN_SCROLL_DELTA: f64 = 64.0;
/// Maximum absolute axis delta accepted for active-screen mouse-drag input.
pub const MAX_SCREEN_DRAG_DELTA: f32 = 4_096.0;
#[cfg(target_arch = "wasm32")]
const SCREEN_INPUT_SCHEMA_VERSION: u32 = 1;
#[cfg(target_arch = "wasm32")]
const HANDLE_BYTES: u32 = 24;

/// Module-owned active-screen input subscription handle.
pub type ScreenInputSubscriptionHandle = EventSubscriptionHandle;

/// Module-owned handle for synchronous active-screen key veto callbacks.
pub type ScreenKeyPressedSubscriptionHandle = EventSubscriptionHandle;

/// Module-owned handle for synchronous active-screen mouse-release veto callbacks.
pub type ScreenMouseReleasedSubscriptionHandle = EventSubscriptionHandle;

/// Module-owned handle for synchronous active-screen mouse-drag veto callbacks.
pub type ScreenMouseDraggedSubscriptionHandle = EventSubscriptionHandle;

/// Detached press data available before the active screen's `keyPressed` handler runs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScreenKeyPressedEvent {
    /// GLFW key code; -1 is an unknown key with a valid scan code.
    pub key_code: i32,
    pub scan_code: u16,
    /// Shift, Control, Alt, Super, Caps Lock, and Num Lock occupy the low six bits.
    pub modifiers: u8,
    /// True when this is a repeated press rather than the initial key-down.
    pub repeat: bool,
}

/// Detached release data captured before the active screen's `mouseReleased` handler runs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenMouseReleasedEvent {
    /// Scaled logical GUI X coordinate.
    pub mouse_x: f32,
    /// Scaled logical GUI Y coordinate.
    pub mouse_y: f32,
    /// GLFW mouse button index.
    pub button: u8,
    /// Shift, Control, Alt, Super, Caps Lock, and Num Lock occupy the low six bits.
    pub modifiers: u8,
}

/// Detached pointer movement captured while a button is held over the active screen.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenMouseDraggedEvent {
    /// Scaled logical GUI X coordinate after movement.
    pub mouse_x: f32,
    /// Scaled logical GUI Y coordinate after movement.
    pub mouse_y: f32,
    /// Scaled logical GUI X movement since the previous drag callback.
    pub delta_x: f32,
    /// Scaled logical GUI Y movement since the previous drag callback.
    pub delta_y: f32,
    /// GLFW mouse button index held during this movement.
    pub button: u8,
}

/// A click, release, or key press from the active screen, captured on the client main thread.
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
    /// Mouse button release coordinates are scaled GUI units. This event is delivered only to
    /// the separate `subscribe_mouse_released` lane, so older subscribers never see a new tag.
    MouseButtonReleased {
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
    /// Fractional mouse-wheel movement over the active screen, in scaled GUI units.
    MouseScrolled {
        mouse_x: f32,
        mouse_y: f32,
        delta_x: f64,
        delta_y: f64,
    },
    /// Active-screen pointer movement. Delivered only to `subscribe_mouse_dragged` callbacks
    /// because older general subscribers do not understand this event tag.
    MouseDragged {
        mouse_x: f32,
        mouse_y: f32,
        delta_x: f32,
        delta_y: f32,
        button: u8,
    },
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
            Self::MouseButtonReleased {
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
            Self::MouseScrolled {
                mouse_x,
                mouse_y,
                delta_x,
                delta_y,
            } => {
                validate_coordinate(mouse_x)?;
                validate_coordinate(mouse_y)?;
                if !delta_x.is_finite()
                    || !delta_y.is_finite()
                    || delta_x.abs() > MAX_SCREEN_SCROLL_DELTA
                    || delta_y.abs() > MAX_SCREEN_SCROLL_DELTA
                    || (delta_x == 0.0 && delta_y == 0.0)
                {
                    return Err(invalid(
                        "screen scroll deltas must be finite, bounded, and non-zero",
                    ));
                }
            }
            Self::MouseDragged {
                mouse_x,
                mouse_y,
                delta_x,
                delta_y,
                button,
            } => {
                validate_coordinate(mouse_x)?;
                validate_coordinate(mouse_y)?;
                if !delta_x.is_finite()
                    || !delta_y.is_finite()
                    || delta_x.abs() > MAX_SCREEN_DRAG_DELTA
                    || delta_y.abs() > MAX_SCREEN_DRAG_DELTA
                {
                    return Err(invalid("screen drag deltas must be finite and bounded"));
                }
                if button > 31 {
                    return Err(invalid("mouse button is outside the supported range"));
                }
            }
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
    #[link_name = "subscribe_key_pressed"]
    fn imported_subscribe_key_pressed(callback_id: i64, handle_output_pointer: i32) -> i32;
    #[link_name = "unsubscribe_key_pressed"]
    fn imported_unsubscribe_key_pressed(handle_pointer: i32, handle_length: i32) -> i32;
    #[link_name = "subscribe_mouse_released"]
    fn imported_subscribe_mouse_released(callback_id: i64, handle_output_pointer: i32) -> i32;
    #[link_name = "unsubscribe_mouse_released"]
    fn imported_unsubscribe_mouse_released(handle_pointer: i32, handle_length: i32) -> i32;
    #[link_name = "subscribe_mouse_dragged"]
    fn imported_subscribe_mouse_dragged(callback_id: i64, handle_output_pointer: i32) -> i32;
    #[link_name = "unsubscribe_mouse_dragged"]
    fn imported_unsubscribe_mouse_dragged(handle_pointer: i32, handle_length: i32) -> i32;
}

#[cfg(target_arch = "wasm32")]
type Callback =
    std::rc::Rc<std::cell::RefCell<Box<dyn FnMut(ScreenInputEvent) -> SjmlResult<bool>>>>;

#[cfg(target_arch = "wasm32")]
struct CallbackEntry {
    callback: Callback,
    handle: Option<ScreenInputSubscriptionHandle>,
    subscription_kind: SubscriptionKind,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SubscriptionKind {
    General,
    KeyPressed,
    MouseReleased,
    MouseDragged,
}

#[cfg(target_arch = "wasm32")]
#[derive(Default)]
struct Callbacks {
    next: Option<u64>,
    entries: std::collections::BTreeMap<u64, CallbackEntry>,
}

#[cfg(target_arch = "wasm32")]
impl Callbacks {
    fn insert<F>(&mut self, callback: F, subscription_kind: SubscriptionKind) -> SjmlResult<u64>
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
                subscription_kind,
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

/// Subscribes a typed callback. Return `true` to consume the mouse, key, character, or scroll event.
pub fn subscribe<F>(callback: F) -> SjmlResult<ScreenInputSubscriptionHandle>
where
    F: FnMut(ScreenInputEvent) -> SjmlResult<bool> + 'static,
{
    #[cfg(target_arch = "wasm32")]
    {
        let callback_id = CALLBACKS.with(|registry| {
            registry
                .borrow_mut()
                .insert(callback, SubscriptionKind::General)
        })?;
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

/// Subscribes a synchronous boolean callback to active-screen key presses only.
/// Return `true` to veto the screen's `keyPressed` handler. Requires
/// `client.input.screen_cancel`; `#[event_async]` handlers cannot be used here.
pub fn subscribe_key_pressed<F>(callback: F) -> SjmlResult<ScreenKeyPressedSubscriptionHandle>
where
    F: FnMut(ScreenKeyPressedEvent) -> SjmlResult<bool> + 'static,
{
    #[cfg(target_arch = "wasm32")]
    {
        let mut callback = callback;
        let adapter = move |event: ScreenInputEvent| match event {
            ScreenInputEvent::KeyPressed {
                key_code,
                scan_code,
                modifiers,
                repeat,
            } => callback(ScreenKeyPressedEvent {
                key_code,
                scan_code,
                modifiers,
                repeat,
            }),
            _ => Ok(false),
        };
        let callback_id = CALLBACKS.with(|registry| {
            registry
                .borrow_mut()
                .insert(adapter, SubscriptionKind::KeyPressed)
        })?;
        let output = allocate_host_input(HANDLE_BYTES);
        if output == 0 {
            CALLBACKS.with(|registry| registry.borrow_mut().entries.remove(&callback_id));
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "could not allocate screen key veto handle output",
            ));
        }
        // SAFETY: initialize the complete live output allocation before the host writes the handle.
        unsafe { core::ptr::write_bytes(output as *mut u8, 0, HANDLE_BYTES as usize) };
        // SAFETY: the exact output allocation remains live during the synchronous host import.
        let status = unsafe { imported_subscribe_key_pressed(callback_id as i64, output as i32) };
        let result = if status == 0 {
            // SAFETY: a successful host call initialized the complete handle allocation.
            from_bytes::<ScreenKeyPressedSubscriptionHandle>(unsafe {
                core::slice::from_raw_parts(output as *const u8, HANDLE_BYTES as usize)
            })
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected screen key veto subscription",
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

/// Subscribes a synchronous callback to active-screen mouse-button releases only.
/// Return `true` to veto the screen's `mouseReleased` handler. Requires
/// `client.input.screen_cancel`; older general screen-input subscribers never receive release events.
pub fn subscribe_mouse_released<F>(callback: F) -> SjmlResult<ScreenMouseReleasedSubscriptionHandle>
where
    F: FnMut(ScreenMouseReleasedEvent) -> SjmlResult<bool> + 'static,
{
    #[cfg(target_arch = "wasm32")]
    {
        let mut callback = callback;
        let adapter = move |event: ScreenInputEvent| match event {
            ScreenInputEvent::MouseButtonReleased {
                mouse_x,
                mouse_y,
                button,
                modifiers,
            } => callback(ScreenMouseReleasedEvent {
                mouse_x,
                mouse_y,
                button,
                modifiers,
            }),
            _ => Ok(false),
        };
        let callback_id = CALLBACKS.with(|registry| {
            registry
                .borrow_mut()
                .insert(adapter, SubscriptionKind::MouseReleased)
        })?;
        let output = allocate_host_input(HANDLE_BYTES);
        if output == 0 {
            CALLBACKS.with(|registry| registry.borrow_mut().entries.remove(&callback_id));
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "could not allocate screen mouse-release veto handle output",
            ));
        }
        // SAFETY: initialize the complete live output allocation before the host writes the handle.
        unsafe { core::ptr::write_bytes(output as *mut u8, 0, HANDLE_BYTES as usize) };
        // SAFETY: the output allocation remains live during the synchronous host import.
        let status =
            unsafe { imported_subscribe_mouse_released(callback_id as i64, output as i32) };
        let result = if status == 0 {
            // SAFETY: a successful host call initialized the complete handle allocation.
            from_bytes::<ScreenMouseReleasedSubscriptionHandle>(unsafe {
                core::slice::from_raw_parts(output as *const u8, HANDLE_BYTES as usize)
            })
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected screen mouse-release subscription",
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

/// Subscribes a synchronous callback to pointer drags over the active screen only.
/// Return `true` to consume the screen's drag handler. Requires
/// `client.input.screen_cancel`; asynchronous callbacks cannot be used here.
pub fn subscribe_mouse_dragged<F>(callback: F) -> SjmlResult<ScreenMouseDraggedSubscriptionHandle>
where
    F: FnMut(ScreenMouseDraggedEvent) -> SjmlResult<bool> + 'static,
{
    #[cfg(target_arch = "wasm32")]
    {
        let mut callback = callback;
        let adapter = move |event: ScreenInputEvent| match event {
            ScreenInputEvent::MouseDragged {
                mouse_x,
                mouse_y,
                delta_x,
                delta_y,
                button,
            } => callback(ScreenMouseDraggedEvent {
                mouse_x,
                mouse_y,
                delta_x,
                delta_y,
                button,
            }),
            _ => Ok(false),
        };
        let callback_id = CALLBACKS.with(|registry| {
            registry
                .borrow_mut()
                .insert(adapter, SubscriptionKind::MouseDragged)
        })?;
        let output = allocate_host_input(HANDLE_BYTES);
        if output == 0 {
            CALLBACKS.with(|registry| registry.borrow_mut().entries.remove(&callback_id));
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "could not allocate screen mouse-drag handle output",
            ));
        }
        // SAFETY: initialize the complete live output allocation before the host writes the handle.
        unsafe { core::ptr::write_bytes(output as *mut u8, 0, HANDLE_BYTES as usize) };
        // SAFETY: the output allocation remains live during the synchronous host import.
        let status = unsafe { imported_subscribe_mouse_dragged(callback_id as i64, output as i32) };
        let result = if status == 0 {
            // SAFETY: a successful host call initialized the complete handle allocation.
            from_bytes::<ScreenMouseDraggedSubscriptionHandle>(unsafe {
                core::slice::from_raw_parts(output as *const u8, HANDLE_BYTES as usize)
            })
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected screen mouse-drag subscription",
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
        let (callback_id, subscription_kind) = CALLBACKS
            .with(|registry| {
                registry.borrow().entries.iter().find_map(|(&id, entry)| {
                    (entry.handle == Some(handle)).then_some((id, entry.subscription_kind))
                })
            })
            .ok_or_else(|| {
                SjmlError::new(
                    ErrorCode::StaleHandle,
                    "screen input subscription is not active in this module",
                )
            })?;
        let bytes = to_bytes(&handle)?;
        // SAFETY: the stable handle bytes live for the complete synchronous host call.
        let status = match subscription_kind {
            SubscriptionKind::General => {
                // SAFETY: the exact serialized handle remains live for the synchronous host call.
                unsafe { imported_unsubscribe(bytes.as_ptr() as i32, bytes.len() as i32) }
            }
            SubscriptionKind::KeyPressed => {
                // SAFETY: the exact serialized handle remains live for the synchronous host call.
                unsafe {
                    imported_unsubscribe_key_pressed(bytes.as_ptr() as i32, bytes.len() as i32)
                }
            }
            SubscriptionKind::MouseReleased => {
                // SAFETY: the exact serialized handle remains live for the synchronous host call.
                unsafe {
                    imported_unsubscribe_mouse_released(bytes.as_ptr() as i32, bytes.len() as i32)
                }
            }
            SubscriptionKind::MouseDragged => {
                // SAFETY: the exact serialized handle remains live for the synchronous host call.
                unsafe {
                    imported_unsubscribe_mouse_dragged(bytes.as_ptr() as i32, bytes.len() as i32)
                }
            }
        };
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

/// Removes the synchronous active-screen key-veto callback returned by
/// [`subscribe_key_pressed`].
pub fn unsubscribe_key_pressed(handle: ScreenKeyPressedSubscriptionHandle) -> SjmlResult<()> {
    unsubscribe(handle)
}

/// Removes the synchronous active-screen mouse-release callback returned by
/// [`subscribe_mouse_released`].
pub fn unsubscribe_mouse_released(handle: ScreenMouseReleasedSubscriptionHandle) -> SjmlResult<()> {
    unsubscribe(handle)
}

/// Removes the synchronous active-screen mouse-drag callback returned by
/// [`subscribe_mouse_dragged`].
pub fn unsubscribe_mouse_dragged(handle: ScreenMouseDraggedSubscriptionHandle) -> SjmlResult<()> {
    unsubscribe(handle)
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
            4 => ScreenInputEvent::MouseButtonReleased {
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
            3 => ScreenInputEvent::MouseScrolled {
                mouse_x: decoder.read_f32()?,
                mouse_y: decoder.read_f32()?,
                delta_x: decoder.read_f64()?,
                delta_y: decoder.read_f64()?,
            },
            5 => ScreenInputEvent::MouseDragged {
                mouse_x: decoder.read_f32()?,
                mouse_y: decoder.read_f32()?,
                delta_x: decoder.read_f32()?,
                delta_y: decoder.read_f32()?,
                button: decoder.read_u8()?,
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
    use super::{
        ScreenInputEvent, ScreenKeyPressedEvent, MAX_SCREEN_DRAG_DELTA,
        MAX_SCREEN_INPUT_COORDINATE, MAX_SCREEN_SCROLL_DELTA,
    };

    #[test]
    fn validates_bounded_click_and_key_events() {
        let click = ScreenInputEvent::MouseButtonPressed {
            mouse_x: MAX_SCREEN_INPUT_COORDINATE,
            mouse_y: -MAX_SCREEN_INPUT_COORDINATE,
            button: 7,
            modifiers: 0x03,
        };
        assert_eq!(click.validate().unwrap(), click);
        let release = ScreenInputEvent::MouseButtonReleased {
            mouse_x: 80.0,
            mouse_y: 30.0,
            button: 1,
            modifiers: 0x02,
        };
        assert_eq!(release.validate().unwrap(), release);
        let key = ScreenInputEvent::KeyPressed {
            key_code: -1,
            scan_code: 65_535,
            modifiers: 0x20,
            repeat: true,
        };
        assert_eq!(key.validate().unwrap(), key);
        let veto = ScreenKeyPressedEvent {
            key_code: -1,
            scan_code: 65_535,
            modifiers: 0x20,
            repeat: true,
        };
        assert_eq!(
            veto,
            ScreenKeyPressedEvent {
                key_code: -1,
                scan_code: 65_535,
                modifiers: 0x20,
                repeat: true,
            }
        );
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
        assert!(ScreenInputEvent::MouseButtonReleased {
            mouse_x: 0.0,
            mouse_y: 0.0,
            button: 32,
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

    #[test]
    fn validates_fractional_bounded_screen_scroll_events() {
        let event = ScreenInputEvent::MouseScrolled {
            mouse_x: 10.5,
            mouse_y: 20.0,
            delta_x: -0.25,
            delta_y: MAX_SCREEN_SCROLL_DELTA,
        };
        assert_eq!(event.validate().unwrap(), event);
        for (delta_x, delta_y) in [
            (f64::NAN, 1.0),
            (0.0, 0.0),
            (MAX_SCREEN_SCROLL_DELTA + 0.1, 0.0),
        ] {
            assert!(ScreenInputEvent::MouseScrolled {
                mouse_x: 0.0,
                mouse_y: 0.0,
                delta_x,
                delta_y,
            }
            .validate()
            .is_err());
        }
    }

    #[test]
    fn validates_bounded_active_screen_mouse_drag_events() {
        let event = ScreenInputEvent::MouseDragged {
            mouse_x: MAX_SCREEN_INPUT_COORDINATE,
            mouse_y: -MAX_SCREEN_INPUT_COORDINATE,
            delta_x: -MAX_SCREEN_DRAG_DELTA,
            delta_y: MAX_SCREEN_DRAG_DELTA,
            button: 31,
        };
        assert_eq!(event.validate().unwrap(), event);
        for (delta_x, delta_y, button) in [
            (f32::NAN, 0.0, 0),
            (0.0, f32::INFINITY, 0),
            (MAX_SCREEN_DRAG_DELTA + 0.1, 0.0, 0),
            (0.0, 0.0, 32),
        ] {
            assert!(ScreenInputEvent::MouseDragged {
                mouse_x: 0.0,
                mouse_y: 0.0,
                delta_x,
                delta_y,
                button,
            }
            .validate()
            .is_err());
        }
    }
}
