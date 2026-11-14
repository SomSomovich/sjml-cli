//! Bounded global keyboard, mouse-button, mouse-wheel, and cursor callbacks for client guests.

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
#[cfg(target_arch = "wasm32")]
const CURSOR_POSITION_BYTES: u32 = 16;
#[cfg(target_arch = "wasm32")]
const INPUT_STATE_BYTES: u32 = 1;
/// Largest GLFW keyboard key code accepted by the held-state query.
pub const MAX_GLFW_KEY_CODE: i32 = 348;
/// Largest GLFW mouse-button index accepted by the held-state query.
pub const MAX_MOUSE_BUTTON: u8 = 7;
/// Largest supported logical GUI coordinate returned by the host.
pub const MAX_CURSOR_COORDINATE: f64 = 1_000_000.0;
/// Largest per-tick sampled cursor delta accepted by the host and SDK.
pub const MAX_CURSOR_MOVED_DELTA: f64 = 4_096.0;
/// Maximum number of active cursor movement listeners across the physical client.
pub const MAX_CURSOR_MOVED_SUBSCRIPTIONS: usize = 256;
/// Maximum number of cursor movement listeners a single module can keep active.
pub const MAX_CURSOR_MOVED_SUBSCRIPTIONS_PER_MODULE: usize = 1;
/// Maximum number of raw cursor-motion listeners across the physical client.
pub const MAX_CURSOR_MOTION_SUBSCRIPTIONS: usize = 64;
/// Maximum raw cursor callbacks delivered to one module during a client tick.
pub const MAX_CURSOR_MOTION_CALLBACKS_PER_MODULE_PER_TICK: usize = 16;
/// Largest raw cursor coordinate in physical window pixels.
pub const MAX_CURSOR_MOTION_COORDINATE: f64 = 1_000_000.0;
/// Largest accepted native cursor delta in one callback.
pub const MAX_CURSOR_MOTION_DELTA: f64 = 4_096.0;
/// Maximum number of raw cursor-motion listeners a single module can keep active.
pub const MAX_CURSOR_MOTION_SUBSCRIPTIONS_PER_MODULE: usize = 1;

/// Current physical-client mouse position in Minecraft's scaled GUI coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CursorPosition {
    /// Horizontal coordinate from the left edge of the logical GUI viewport.
    pub x: f64,
    /// Vertical coordinate from the top edge of the logical GUI viewport.
    pub y: f64,
}

impl CursorPosition {
    #[cfg(any(target_arch = "wasm32", test))]
    fn validate(self) -> SjmlResult<Self> {
        if !self.x.is_finite()
            || !self.y.is_finite()
            || self.x < 0.0
            || self.y < 0.0
            || self.x > MAX_CURSOR_COORDINATE
            || self.y > MAX_CURSOR_COORDINATE
        {
            return Err(SjmlError::new(
                ErrorCode::InvalidTransfer,
                "host cursor coordinates are not finite or exceed the supported GUI bounds",
            ));
        }
        Ok(self)
    }
}

/// Module-owned global client-input subscription handle.
pub type ClientInputSubscriptionHandle = EventSubscriptionHandle;

/// Module-owned subscription handle for observe-only global keyboard notifications.
pub type GlobalKeyObservationHandle = EventSubscriptionHandle;

/// Module-owned global mouse-wheel subscription handle.
pub type MouseScrollSubscriptionHandle = EventSubscriptionHandle;

/// Module-owned subscription handle for sampled cursor movement.
pub type CursorMovedSubscriptionHandle = EventSubscriptionHandle;

/// Module-owned subscription handle for raw physical-client cursor motion.
pub type CursorMotionSubscriptionHandle = EventSubscriptionHandle;

/// Bounded horizontal and vertical scroll movement reported outside screens on the client.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MouseScrollEvent {
    pub delta_x: f64,
    pub delta_y: f64,
}

impl MouseScrollEvent {
    pub const MAX_DELTA: f64 = 64.0;

    pub fn new(delta_x: f64, delta_y: f64) -> SjmlResult<Self> {
        Self { delta_x, delta_y }.validate()
    }

    fn validate(self) -> SjmlResult<Self> {
        if !self.delta_x.is_finite()
            || !self.delta_y.is_finite()
            || self.delta_x.abs() > Self::MAX_DELTA
            || self.delta_y.abs() > Self::MAX_DELTA
            || (self.delta_x == 0.0 && self.delta_y == 0.0)
        {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "mouse scroll deltas must be finite, bounded, and non-zero",
            ));
        }
        Ok(self)
    }
}

/// One non-zero GUI-coordinate movement sampled at most once during a client tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientCursorMovedEvent {
    /// Current horizontal cursor position in logical GUI coordinates.
    pub x: f64,
    /// Current vertical cursor position in logical GUI coordinates.
    pub y: f64,
    /// Horizontal change since the previous client tick sample.
    pub delta_x: f64,
    /// Vertical change since the previous client tick sample.
    pub delta_y: f64,
}

impl ClientCursorMovedEvent {
    pub const MAX_DELTA: f64 = MAX_CURSOR_MOVED_DELTA;

    pub fn new(x: f64, y: f64, delta_x: f64, delta_y: f64) -> SjmlResult<Self> {
        Self {
            x,
            y,
            delta_x,
            delta_y,
        }
        .validate()
    }

    fn validate(self) -> SjmlResult<Self> {
        if !self.x.is_finite()
            || !self.y.is_finite()
            || self.x < 0.0
            || self.y < 0.0
            || self.x > MAX_CURSOR_COORDINATE
            || self.y > MAX_CURSOR_COORDINATE
            || !self.delta_x.is_finite()
            || !self.delta_y.is_finite()
            || self.delta_x.abs() > Self::MAX_DELTA
            || self.delta_y.abs() > Self::MAX_DELTA
            || (self.delta_x == 0.0 && self.delta_y == 0.0)
        {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "cursor movement coordinates and non-zero deltas must be finite and bounded",
            ));
        }
        Ok(self)
    }
}

/// One native cursor-position event in physical window pixels, distinct from tick-sampled GUI motion.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientCursorMotionEvent {
    /// Current horizontal position from the physical window's left edge, in window pixels.
    pub x: f64,
    /// Current vertical position from the physical window's top edge, in window pixels.
    pub y: f64,
    /// Horizontal movement since the previous GLFW cursor callback.
    pub delta_x: f64,
    /// Vertical movement since the previous GLFW cursor callback.
    pub delta_y: f64,
}

impl ClientCursorMotionEvent {
    pub const MAX_DELTA: f64 = MAX_CURSOR_MOTION_DELTA;

    pub fn new(x: f64, y: f64, delta_x: f64, delta_y: f64) -> SjmlResult<Self> {
        Self {
            x,
            y,
            delta_x,
            delta_y,
        }
        .validate()
    }

    fn validate(self) -> SjmlResult<Self> {
        if !self.x.is_finite()
            || !self.y.is_finite()
            || self.x < 0.0
            || self.y < 0.0
            || self.x > MAX_CURSOR_MOTION_COORDINATE
            || self.y > MAX_CURSOR_MOTION_COORDINATE
            || !self.delta_x.is_finite()
            || !self.delta_y.is_finite()
            || self.delta_x.abs() > Self::MAX_DELTA
            || self.delta_y.abs() > Self::MAX_DELTA
            || (self.delta_x == 0.0 && self.delta_y == 0.0)
        {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "raw cursor motion coordinates and non-zero deltas must be finite and bounded",
            ));
        }
        Ok(self)
    }
}

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

/// Detached key data posted by NeoForge after vanilla key mappings have already run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GlobalKeyObservation {
    pub key_code: i32,
    pub scan_code: u16,
    pub action: ClientInputAction,
    pub modifiers: u8,
}

#[cfg(any(target_arch = "wasm32", test))]
impl GlobalKeyObservation {
    fn from_input(event: ClientInputEvent) -> Option<Self> {
        match event {
            ClientInputEvent::Key {
                key_code,
                scan_code,
                action,
                modifiers,
            } => Some(Self {
                key_code,
                scan_code,
                action,
                modifiers,
            }),
            ClientInputEvent::MouseButton { .. } => None,
        }
    }
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
    #[link_name = "subscribe_key_observation"]
    fn imported_subscribe_key_observation(callback_id: i64, handle_output_pointer: i32) -> i32;
    #[link_name = "unsubscribe_key_observation"]
    fn imported_unsubscribe_key_observation(handle_pointer: i32, handle_length: i32) -> i32;
    #[link_name = "subscribe_scroll"]
    fn imported_subscribe_scroll(callback_id: i64, handle_output_pointer: i32) -> i32;
    #[link_name = "unsubscribe_scroll"]
    fn imported_unsubscribe_scroll(handle_pointer: i32, handle_length: i32) -> i32;
    #[link_name = "subscribe_cursor_moved"]
    fn imported_subscribe_cursor_moved(callback_id: i64, handle_output_pointer: i32) -> i32;
    #[link_name = "unsubscribe_cursor_moved"]
    fn imported_unsubscribe_cursor_moved(handle_pointer: i32, handle_length: i32) -> i32;
    #[link_name = "subscribe_cursor_motion"]
    fn imported_subscribe_cursor_motion(callback_id: i64, handle_output_pointer: i32) -> i32;
    #[link_name = "unsubscribe_cursor_motion"]
    fn imported_unsubscribe_cursor_motion(handle_pointer: i32, handle_length: i32) -> i32;
    #[link_name = "cursor_position"]
    fn imported_cursor_position(output_pointer: i32) -> i32;
    #[link_name = "is_key_down"]
    fn imported_is_key_down(key_code: i32, output_pointer: i32) -> i32;
    #[link_name = "is_mouse_button_down"]
    fn imported_is_mouse_button_down(button: i32, output_pointer: i32) -> i32;
}

#[cfg(target_arch = "wasm32")]
type Callback =
    std::rc::Rc<std::cell::RefCell<Box<dyn FnMut(ClientInputEvent) -> SjmlResult<bool>>>>;

#[cfg(target_arch = "wasm32")]
struct CallbackEntry {
    callback: Callback,
    handle: Option<ClientInputSubscriptionHandle>,
    key_observation_only: bool,
}

#[cfg(target_arch = "wasm32")]
#[derive(Default)]
struct Callbacks {
    next: Option<u64>,
    entries: std::collections::BTreeMap<u64, CallbackEntry>,
}

#[cfg(target_arch = "wasm32")]
impl Callbacks {
    fn insert<F>(&mut self, callback: F, key_observation_only: bool) -> SjmlResult<u64>
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
                key_observation_only,
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
type MouseScrollCallback =
    std::rc::Rc<std::cell::RefCell<Box<dyn FnMut(MouseScrollEvent) -> SjmlResult<()>>>>;

#[cfg(target_arch = "wasm32")]
struct MouseScrollCallbackEntry {
    callback: MouseScrollCallback,
    handle: Option<MouseScrollSubscriptionHandle>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Default)]
struct MouseScrollCallbacks {
    next: Option<u64>,
    entries: std::collections::BTreeMap<u64, MouseScrollCallbackEntry>,
}

#[cfg(target_arch = "wasm32")]
impl MouseScrollCallbacks {
    fn insert<F>(&mut self, callback: F) -> SjmlResult<u64>
    where
        F: FnMut(MouseScrollEvent) -> SjmlResult<()> + 'static,
    {
        if self.entries.len() >= MAX_CLIENT_INPUT_SUBSCRIPTIONS {
            return Err(SjmlError::new(
                ErrorCode::RegistrationLimitReached,
                "mouse scroll subscription limit reached",
            ));
        }
        let id = self.next.unwrap_or(1);
        self.next = id.checked_add(1);
        self.entries.insert(
            id,
            MouseScrollCallbackEntry {
                callback: std::rc::Rc::new(std::cell::RefCell::new(Box::new(callback))),
                handle: None,
            },
        );
        Ok(id)
    }

    fn get(&self, id: u64) -> SjmlResult<MouseScrollCallback> {
        self.entries
            .get(&id)
            .map(|entry| entry.callback.clone())
            .ok_or_else(|| {
                SjmlError::new(
                    ErrorCode::StaleHandle,
                    "mouse scroll callback is not subscribed",
                )
            })
    }
}

#[cfg(target_arch = "wasm32")]
type CursorMovedCallback =
    std::rc::Rc<std::cell::RefCell<Box<dyn FnMut(ClientCursorMovedEvent) -> SjmlResult<()>>>>;

#[cfg(target_arch = "wasm32")]
struct CursorMovedCallbackEntry {
    callback: CursorMovedCallback,
    handle: Option<CursorMovedSubscriptionHandle>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Default)]
struct CursorMovedCallbacks {
    next: Option<u64>,
    entries: std::collections::BTreeMap<u64, CursorMovedCallbackEntry>,
}

#[cfg(target_arch = "wasm32")]
impl CursorMovedCallbacks {
    fn insert<F>(&mut self, callback: F) -> SjmlResult<u64>
    where
        F: FnMut(ClientCursorMovedEvent) -> SjmlResult<()> + 'static,
    {
        if self.entries.len() >= MAX_CURSOR_MOVED_SUBSCRIPTIONS_PER_MODULE {
            return Err(SjmlError::new(
                ErrorCode::RegistrationLimitReached,
                "a module can have only one cursor movement listener",
            ));
        }
        let id = self.next.unwrap_or(1);
        self.next = id.checked_add(1);
        self.entries.insert(
            id,
            CursorMovedCallbackEntry {
                callback: std::rc::Rc::new(std::cell::RefCell::new(Box::new(callback))),
                handle: None,
            },
        );
        Ok(id)
    }

    fn get(&self, id: u64) -> SjmlResult<CursorMovedCallback> {
        self.entries
            .get(&id)
            .map(|entry| entry.callback.clone())
            .ok_or_else(|| {
                SjmlError::new(
                    ErrorCode::StaleHandle,
                    "cursor movement callback is not subscribed",
                )
            })
    }
}

#[cfg(target_arch = "wasm32")]
type CursorMotionCallback =
    std::rc::Rc<std::cell::RefCell<Box<dyn FnMut(ClientCursorMotionEvent) -> SjmlResult<()>>>>;

#[cfg(target_arch = "wasm32")]
struct CursorMotionCallbackEntry {
    callback: CursorMotionCallback,
    handle: Option<CursorMotionSubscriptionHandle>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Default)]
struct CursorMotionCallbacks {
    next: Option<u64>,
    entries: std::collections::BTreeMap<u64, CursorMotionCallbackEntry>,
}

#[cfg(target_arch = "wasm32")]
impl CursorMotionCallbacks {
    fn insert<F>(&mut self, callback: F) -> SjmlResult<u64>
    where
        F: FnMut(ClientCursorMotionEvent) -> SjmlResult<()> + 'static,
    {
        if self.entries.len() >= MAX_CURSOR_MOTION_SUBSCRIPTIONS_PER_MODULE {
            return Err(SjmlError::new(
                ErrorCode::RegistrationLimitReached,
                "a module can have only one raw cursor-motion listener",
            ));
        }
        let id = self.next.unwrap_or(1);
        self.next = id.checked_add(1);
        self.entries.insert(
            id,
            CursorMotionCallbackEntry {
                callback: std::rc::Rc::new(std::cell::RefCell::new(Box::new(callback))),
                handle: None,
            },
        );
        Ok(id)
    }

    fn get(&self, id: u64) -> SjmlResult<CursorMotionCallback> {
        self.entries
            .get(&id)
            .map(|entry| entry.callback.clone())
            .ok_or_else(|| {
                SjmlError::new(
                    ErrorCode::StaleHandle,
                    "cursor motion callback is not subscribed",
                )
            })
    }
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static CALLBACKS: std::cell::RefCell<Callbacks> = std::cell::RefCell::new(Callbacks::default());
    static MOUSE_SCROLL_CALLBACKS: std::cell::RefCell<MouseScrollCallbacks> =
        std::cell::RefCell::new(MouseScrollCallbacks::default());
    static CURSOR_MOVED_CALLBACKS: std::cell::RefCell<CursorMovedCallbacks> =
        std::cell::RefCell::new(CursorMovedCallbacks::default());
    static CURSOR_MOTION_CALLBACKS: std::cell::RefCell<CursorMotionCallbacks> =
        std::cell::RefCell::new(CursorMotionCallbacks::default());
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
        let callback_id =
            CALLBACKS.with(|registry| registry.borrow_mut().insert(callback, false))?;
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

/// Subscribes to bounded global keyboard notices under `client.input.observe`.
/// NeoForge emits these after vanilla key mappings and Minecraft processing; returning a value or
/// cancelling this callback cannot change key handling. Use `screen_input::subscribe_key_pressed`
/// for synchronous veto of key presses delivered to an already open screen.
pub fn subscribe_global_keys<F>(callback: F) -> SjmlResult<GlobalKeyObservationHandle>
where
    F: FnMut(GlobalKeyObservation) -> SjmlResult<()> + 'static,
{
    #[cfg(target_arch = "wasm32")]
    {
        let mut callback = callback;
        let adapter = move |event: ClientInputEvent| {
            if let Some(event) = GlobalKeyObservation::from_input(event) {
                callback(event)?;
            }
            Ok(false)
        };
        let callback_id = CALLBACKS.with(|registry| registry.borrow_mut().insert(adapter, true))?;
        let output = allocate_host_input(HANDLE_BYTES);
        if output == 0 {
            CALLBACKS.with(|registry| registry.borrow_mut().entries.remove(&callback_id));
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "could not allocate global key observation handle output",
            ));
        }
        // SAFETY: initialize the complete live output allocation before the host writes the handle.
        unsafe { core::ptr::write_bytes(output as *mut u8, 0, HANDLE_BYTES as usize) };
        // SAFETY: the exact output allocation remains live during the synchronous host import.
        let status =
            unsafe { imported_subscribe_key_observation(callback_id as i64, output as i32) };
        let result = if status == 0 {
            // SAFETY: a successful call initialized the full handle allocation.
            from_bytes::<GlobalKeyObservationHandle>(unsafe {
                core::slice::from_raw_parts(output as *const u8, HANDLE_BYTES as usize)
            })
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected global key observation subscription",
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
        let (callback_id, key_observation_only) = CALLBACKS
            .with(|registry| {
                registry.borrow().entries.iter().find_map(|(&id, entry)| {
                    (entry.handle == Some(handle)).then_some((id, entry.key_observation_only))
                })
            })
            .ok_or_else(|| {
                SjmlError::new(
                    ErrorCode::StaleHandle,
                    "client input subscription is not active in this module",
                )
            })?;
        let bytes = to_bytes(&handle)?;
        // SAFETY: the stable handle bytes live for the complete synchronous host call.
        let status = if key_observation_only {
            // SAFETY: the exact serialized handle remains live for the synchronous host call.
            unsafe {
                imported_unsubscribe_key_observation(bytes.as_ptr() as i32, bytes.len() as i32)
            }
        } else {
            // SAFETY: the exact serialized handle remains live for the synchronous host call.
            unsafe { imported_unsubscribe(bytes.as_ptr() as i32, bytes.len() as i32) }
        };
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

/// Removes a global-key observer using the same module-scoped handle returned by
/// [`subscribe_global_keys`].
pub fn unsubscribe_global_keys(handle: GlobalKeyObservationHandle) -> SjmlResult<()> {
    unsubscribe(handle)
}

/// Subscribes to horizontal and vertical mouse-wheel movement. The event is observational: the
/// callback cannot consume or rewrite vanilla scroll input, and it runs only on the physical
/// client's main thread outside screens.
pub fn subscribe_scroll<F>(callback: F) -> SjmlResult<MouseScrollSubscriptionHandle>
where
    F: FnMut(MouseScrollEvent) -> SjmlResult<()> + 'static,
{
    #[cfg(target_arch = "wasm32")]
    {
        let callback_id =
            MOUSE_SCROLL_CALLBACKS.with(|registry| registry.borrow_mut().insert(callback))?;
        let output = allocate_host_input(HANDLE_BYTES);
        if output == 0 {
            MOUSE_SCROLL_CALLBACKS
                .with(|registry| registry.borrow_mut().entries.remove(&callback_id));
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "could not allocate mouse scroll subscription output",
            ));
        }
        // SAFETY: initialize the complete live output allocation before the host writes the handle.
        unsafe { core::ptr::write_bytes(output as *mut u8, 0, HANDLE_BYTES as usize) };
        // SAFETY: the exact output allocation remains live during the synchronous host import.
        let status = unsafe { imported_subscribe_scroll(callback_id as i64, output as i32) };
        let result = if status == 0 {
            // SAFETY: a successful call initialized the complete handle allocation.
            from_bytes::<MouseScrollSubscriptionHandle>(unsafe {
                core::slice::from_raw_parts(output as *const u8, HANDLE_BYTES as usize)
            })
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected mouse scroll subscription",
            ))
        };
        // SAFETY: release the exact allocation passed to the host import.
        unsafe { deallocate_host_input(output, HANDLE_BYTES) };
        match result {
            Ok(handle) => {
                MOUSE_SCROLL_CALLBACKS.with(|registry| {
                    if let Some(entry) = registry.borrow_mut().entries.get_mut(&callback_id) {
                        entry.handle = Some(handle);
                    }
                });
                Ok(handle)
            }
            Err(error) => {
                MOUSE_SCROLL_CALLBACKS
                    .with(|registry| registry.borrow_mut().entries.remove(&callback_id));
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

/// Removes an active mouse-wheel subscription from the host and guest callback registry.
pub fn unsubscribe_scroll(handle: MouseScrollSubscriptionHandle) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let callback_id = MOUSE_SCROLL_CALLBACKS
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
                    "mouse scroll subscription is not active in this module",
                )
            })?;
        let bytes = to_bytes(&handle)?;
        // SAFETY: the stable handle bytes live for the complete synchronous host call.
        let status =
            unsafe { imported_unsubscribe_scroll(bytes.as_ptr() as i32, bytes.len() as i32) };
        if status != 0 {
            return Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected mouse scroll unsubscription",
            ));
        }
        MOUSE_SCROLL_CALLBACKS.with(|registry| {
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

/// Subscribes to cursor movement sampled once per client tick. This is a bounded sample, not a raw
/// mouse-motion event; it requires `client.input.observe` and runs on the client main thread.
/// Baseline changes after screen transitions, window resizes, or missing cursor context are silent.
pub fn subscribe_cursor_moved<F>(callback: F) -> SjmlResult<CursorMovedSubscriptionHandle>
where
    F: FnMut(ClientCursorMovedEvent) -> SjmlResult<()> + 'static,
{
    #[cfg(target_arch = "wasm32")]
    {
        let callback_id =
            CURSOR_MOVED_CALLBACKS.with(|registry| registry.borrow_mut().insert(callback))?;
        let output = allocate_host_input(HANDLE_BYTES);
        if output == 0 {
            CURSOR_MOVED_CALLBACKS
                .with(|registry| registry.borrow_mut().entries.remove(&callback_id));
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "could not allocate cursor movement subscription output",
            ));
        }
        // SAFETY: initialize the complete live output allocation before the host writes the handle.
        unsafe { core::ptr::write_bytes(output as *mut u8, 0, HANDLE_BYTES as usize) };
        // SAFETY: the exact output allocation remains live during the synchronous host import.
        let status = unsafe { imported_subscribe_cursor_moved(callback_id as i64, output as i32) };
        let result = if status == 0 {
            // SAFETY: a successful call initialized the complete handle allocation.
            from_bytes::<CursorMovedSubscriptionHandle>(unsafe {
                core::slice::from_raw_parts(output as *const u8, HANDLE_BYTES as usize)
            })
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected cursor movement subscription",
            ))
        };
        // SAFETY: release the exact allocation passed to the host import.
        unsafe { deallocate_host_input(output, HANDLE_BYTES) };
        match result {
            Ok(handle) => {
                CURSOR_MOVED_CALLBACKS.with(|registry| {
                    if let Some(entry) = registry.borrow_mut().entries.get_mut(&callback_id) {
                        entry.handle = Some(handle);
                    }
                });
                Ok(handle)
            }
            Err(error) => {
                CURSOR_MOVED_CALLBACKS
                    .with(|registry| registry.borrow_mut().entries.remove(&callback_id));
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

/// Unsubscribes the module's sampled cursor-movement listener.
pub fn unsubscribe_cursor_moved(handle: CursorMovedSubscriptionHandle) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let callback_id = CURSOR_MOVED_CALLBACKS
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
                    "cursor movement subscription is not active in this module",
                )
            })?;
        let bytes = to_bytes(&handle)?;
        // SAFETY: the stable handle bytes live for the complete synchronous host call.
        let status =
            unsafe { imported_unsubscribe_cursor_moved(bytes.as_ptr() as i32, bytes.len() as i32) };
        if status != 0 {
            return Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected cursor movement unsubscription",
            ));
        }
        CURSOR_MOVED_CALLBACKS.with(|registry| {
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

/// Subscribes to each raw native cursor-position callback on the physical client.
///
/// Positions and deltas use physical window pixels. This is independent of
/// [`subscribe_cursor_moved`], which reports at most one scaled GUI-coordinate sample per tick.
/// At most 16 events per module are dispatched in one client tick. Requires the explicit
/// `client.input.cursor_motion` capability; callbacks run synchronously on the client main thread.
pub fn subscribe_cursor_motion<F>(callback: F) -> SjmlResult<CursorMotionSubscriptionHandle>
where
    F: FnMut(ClientCursorMotionEvent) -> SjmlResult<()> + 'static,
{
    #[cfg(target_arch = "wasm32")]
    {
        let callback_id =
            CURSOR_MOTION_CALLBACKS.with(|registry| registry.borrow_mut().insert(callback))?;
        let output = allocate_host_input(HANDLE_BYTES);
        if output == 0 {
            CURSOR_MOTION_CALLBACKS
                .with(|registry| registry.borrow_mut().entries.remove(&callback_id));
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "could not allocate raw cursor-motion subscription output",
            ));
        }
        // SAFETY: initialize the complete live output allocation before the host writes the handle.
        unsafe { core::ptr::write_bytes(output as *mut u8, 0, HANDLE_BYTES as usize) };
        // SAFETY: the exact output allocation remains live during the synchronous host import.
        let status = unsafe { imported_subscribe_cursor_motion(callback_id as i64, output as i32) };
        let result = if status == 0 {
            // SAFETY: a successful host call initialized the complete handle allocation.
            from_bytes::<CursorMotionSubscriptionHandle>(unsafe {
                core::slice::from_raw_parts(output as *const u8, HANDLE_BYTES as usize)
            })
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected raw cursor-motion subscription",
            ))
        };
        // SAFETY: release the exact allocation passed to the host import.
        unsafe { deallocate_host_input(output, HANDLE_BYTES) };
        match result {
            Ok(handle) => {
                CURSOR_MOTION_CALLBACKS.with(|registry| {
                    if let Some(entry) = registry.borrow_mut().entries.get_mut(&callback_id) {
                        entry.handle = Some(handle);
                    }
                });
                Ok(handle)
            }
            Err(error) => {
                CURSOR_MOTION_CALLBACKS
                    .with(|registry| registry.borrow_mut().entries.remove(&callback_id));
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

/// Unsubscribes a raw physical-client cursor-motion listener.
pub fn unsubscribe_cursor_motion(handle: CursorMotionSubscriptionHandle) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let callback_id = CURSOR_MOTION_CALLBACKS
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
                    "raw cursor-motion subscription is not active in this module",
                )
            })?;
        let bytes = to_bytes(&handle)?;
        // SAFETY: the stable handle bytes live for the complete synchronous host call.
        let status = unsafe {
            imported_unsubscribe_cursor_motion(bytes.as_ptr() as i32, bytes.len() as i32)
        };
        if status != 0 {
            return Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected raw cursor-motion unsubscription",
            ));
        }
        CURSOR_MOTION_CALLBACKS.with(|registry| {
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

/// Reads the physical mouse position in scaled GUI coordinates. The origin is the GUI viewport's
/// upper-left corner and coordinates are measured in logical GUI pixels, matching Minecraft
/// screen/widget coordinates. Requires a client module granted `client.input` and a live window.
pub fn cursor_position() -> SjmlResult<CursorPosition> {
    #[cfg(target_arch = "wasm32")]
    {
        let output = allocate_host_input(CURSOR_POSITION_BYTES);
        if output == 0 {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "could not allocate cursor-position output",
            ));
        }
        // SAFETY: initialize the exact output span before the host writes it.
        unsafe { core::ptr::write_bytes(output as *mut u8, 0, CURSOR_POSITION_BYTES as usize) };
        // SAFETY: the output allocation remains live during the synchronous host import.
        let status = unsafe { imported_cursor_position(output as i32) };
        let result = if status == 0 {
            // SAFETY: a successful call initialized both output doubles.
            decode_cursor_position(unsafe {
                core::slice::from_raw_parts(output as *const u8, CURSOR_POSITION_BYTES as usize)
            })
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host could not read the physical client cursor position",
            ))
        };
        // SAFETY: release the exact allocation passed to the host import.
        unsafe { deallocate_host_input(output, CURSOR_POSITION_BYTES) };
        result
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported())
    }
}

/// Reads whether a GLFW keyboard key is currently held on the physical client.
///
/// Key codes are bounded to `0..=348`; scan codes are intentionally not accepted. This synchronous
/// poll requires a client module with `client.input.observe` and runs only on the client main thread.
/// It reports held state without intercepting or changing input handling.
pub fn is_key_down(key_code: i32) -> SjmlResult<bool> {
    if !(0..=MAX_GLFW_KEY_CODE).contains(&key_code) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "GLFW key code must be between 0 and 348",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let output = allocate_host_input(INPUT_STATE_BYTES);
        if output == 0 {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "could not allocate key-state output",
            ));
        }
        // SAFETY: initialize the exact output byte before the synchronous host call.
        unsafe { core::ptr::write_bytes(output as *mut u8, 0, INPUT_STATE_BYTES as usize) };
        // SAFETY: the output allocation remains live during the host import.
        let status = unsafe { imported_is_key_down(key_code, output as i32) };
        let result = if status == 0 {
            // SAFETY: a successful call initializes exactly one boolean byte.
            decode_input_state(unsafe {
                core::slice::from_raw_parts(output as *const u8, INPUT_STATE_BYTES as usize)
            })
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host could not read the physical client key state",
            ))
        };
        // SAFETY: release the exact allocation passed to the host.
        unsafe { deallocate_host_input(output, INPUT_STATE_BYTES) };
        result
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported())
    }
}

/// Reads whether a GLFW mouse button is currently held on the physical client.
///
/// Button indices are bounded to `0..=7`. This synchronous poll requires a client module with
/// `client.input.observe` and runs only on the client main thread. It reports held state without
/// intercepting or changing input handling.
pub fn is_mouse_button_down(button: u8) -> SjmlResult<bool> {
    if button > MAX_MOUSE_BUTTON {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "GLFW mouse button must be between 0 and 7",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let output = allocate_host_input(INPUT_STATE_BYTES);
        if output == 0 {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "could not allocate mouse-button-state output",
            ));
        }
        // SAFETY: initialize the exact output byte before the synchronous host call.
        unsafe { core::ptr::write_bytes(output as *mut u8, 0, INPUT_STATE_BYTES as usize) };
        // SAFETY: the output allocation remains live during the host import.
        let status = unsafe { imported_is_mouse_button_down(i32::from(button), output as i32) };
        let result = if status == 0 {
            // SAFETY: a successful call initializes exactly one boolean byte.
            decode_input_state(unsafe {
                core::slice::from_raw_parts(output as *const u8, INPUT_STATE_BYTES as usize)
            })
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host could not read the physical client mouse-button state",
            ))
        };
        // SAFETY: release the exact allocation passed to the host.
        unsafe { deallocate_host_input(output, INPUT_STATE_BYTES) };
        result
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported())
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn decode_input_state(bytes: &[u8]) -> SjmlResult<bool> {
    match bytes {
        [0] => Ok(false),
        [1] => Ok(true),
        _ => Err(SjmlError::new(
            ErrorCode::InvalidTransfer,
            "held input state must be one boolean byte",
        )),
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn decode_cursor_position(bytes: &[u8]) -> SjmlResult<CursorPosition> {
    if bytes.len() != 16 {
        return Err(SjmlError::new(
            ErrorCode::InvalidTransfer,
            "cursor-position output must contain exactly two doubles",
        ));
    }
    let position = CursorPosition {
        x: f64::from_le_bytes(bytes[0..8].try_into().expect("fixed cursor X span")),
        y: f64::from_le_bytes(bytes[8..16].try_into().expect("fixed cursor Y span")),
    };
    position.validate()
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

#[cfg(target_arch = "wasm32")]
struct MouseScrollDelivery {
    callback_id: u64,
    event: MouseScrollEvent,
}

#[cfg(target_arch = "wasm32")]
impl StableDecode for MouseScrollDelivery {
    fn decode(decoder: &mut crate::Decoder<'_>) -> SjmlResult<Self> {
        if decoder.read_u32()? != CLIENT_INPUT_SCHEMA_VERSION {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                "unsupported mouse scroll dispatch schema",
            ));
        }
        let callback_id = decoder.read_u64()?;
        if callback_id == 0 {
            return Err(invalid("mouse scroll callback identity is zero"));
        }
        let event = MouseScrollEvent::new(decoder.read_f64()?, decoder.read_f64()?)?;
        Ok(Self { callback_id, event })
    }
}

#[cfg(target_arch = "wasm32")]
impl crate::StableEncode for MouseScrollDelivery {
    fn encode(&self, encoder: &mut crate::Encoder) -> SjmlResult<()> {
        encoder.write_u32(CLIENT_INPUT_SCHEMA_VERSION)?;
        encoder.write_u64(self.callback_id)?;
        encoder.write_f64(self.event.delta_x)?;
        encoder.write_f64(self.event.delta_y)
    }
}

/// Dispatches one observational mouse-wheel callback and returns an internal non-consuming bool.
///
/// # Safety
/// The host must pass a live initialized guest allocation and keep it alive until return, then
/// release the input and response with the paired ABI dealloc exports.
#[cfg(target_arch = "wasm32")]
#[export_name = "sjml_v1_client_mouse_scroll_dispatch"]
pub unsafe extern "C" fn dispatch_scroll(pointer: u32, length: u32) -> u64 {
    // SAFETY: the host guarantees the input allocation's initialized live span.
    unsafe {
        crate::abi::dispatch_payload(pointer, length, |delivery: MouseScrollDelivery| {
            if DISPATCHING.with(|active| active.replace(true)) {
                return Err(SjmlError::new(
                    ErrorCode::ServiceBusy,
                    "reentrant client input dispatch is forbidden",
                ));
            }
            let result = (|| {
                let callback = MOUSE_SCROLL_CALLBACKS
                    .with(|registry| registry.borrow().get(delivery.callback_id))?;
                let mut callback = callback.try_borrow_mut().map_err(|_| {
                    SjmlError::new(
                        ErrorCode::ServiceBusy,
                        "mouse scroll callback is already active",
                    )
                })?;
                callback(delivery.event)?;
                Ok(false)
            })();
            DISPATCHING.with(|active| active.set(false));
            result
        })
    }
}

#[cfg(target_arch = "wasm32")]
struct CursorMovedDelivery {
    callback_id: u64,
    event: ClientCursorMovedEvent,
}

#[cfg(target_arch = "wasm32")]
impl StableDecode for CursorMovedDelivery {
    fn decode(decoder: &mut crate::Decoder<'_>) -> SjmlResult<Self> {
        if decoder.read_u32()? != CLIENT_INPUT_SCHEMA_VERSION {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                "unsupported cursor movement dispatch schema",
            ));
        }
        let callback_id = decoder.read_u64()?;
        if callback_id == 0 {
            return Err(invalid("cursor movement callback identity is zero"));
        }
        let event = ClientCursorMovedEvent::new(
            decoder.read_f64()?,
            decoder.read_f64()?,
            decoder.read_f64()?,
            decoder.read_f64()?,
        )?;
        Ok(Self { callback_id, event })
    }
}

#[cfg(target_arch = "wasm32")]
impl crate::StableEncode for CursorMovedDelivery {
    fn encode(&self, encoder: &mut crate::Encoder) -> SjmlResult<()> {
        encoder.write_u32(CLIENT_INPUT_SCHEMA_VERSION)?;
        encoder.write_u64(self.callback_id)?;
        encoder.write_f64(self.event.x)?;
        encoder.write_f64(self.event.y)?;
        encoder.write_f64(self.event.delta_x)?;
        encoder.write_f64(self.event.delta_y)
    }
}

/// Dispatches one sampled, observational cursor movement callback.
///
/// # Safety
/// The host must pass a live initialized guest allocation and keep it alive until return, then
/// release the input and response with the paired ABI dealloc exports.
#[cfg(target_arch = "wasm32")]
#[export_name = "sjml_v1_client_cursor_moved_dispatch"]
pub unsafe extern "C" fn dispatch_cursor_moved(pointer: u32, length: u32) -> u64 {
    // SAFETY: the host guarantees the input allocation's initialized live span.
    unsafe {
        crate::abi::dispatch_payload(pointer, length, |delivery: CursorMovedDelivery| {
            if DISPATCHING.with(|active| active.replace(true)) {
                return Err(SjmlError::new(
                    ErrorCode::ServiceBusy,
                    "reentrant client input dispatch is forbidden",
                ));
            }
            let result = (|| {
                let callback = CURSOR_MOVED_CALLBACKS
                    .with(|registry| registry.borrow().get(delivery.callback_id))?;
                let mut callback = callback.try_borrow_mut().map_err(|_| {
                    SjmlError::new(
                        ErrorCode::ServiceBusy,
                        "cursor movement callback is already active",
                    )
                })?;
                callback(delivery.event)?;
                Ok(false)
            })();
            DISPATCHING.with(|active| active.set(false));
            result
        })
    }
}

#[cfg(target_arch = "wasm32")]
struct CursorMotionDelivery {
    callback_id: u64,
    event: ClientCursorMotionEvent,
}

#[cfg(target_arch = "wasm32")]
impl StableDecode for CursorMotionDelivery {
    fn decode(decoder: &mut crate::Decoder<'_>) -> SjmlResult<Self> {
        if decoder.read_u32()? != CLIENT_INPUT_SCHEMA_VERSION {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                "unsupported raw cursor-motion dispatch schema",
            ));
        }
        let callback_id = decoder.read_u64()?;
        if callback_id == 0 {
            return Err(invalid("raw cursor-motion callback identity is zero"));
        }
        let event = ClientCursorMotionEvent::new(
            decoder.read_f64()?,
            decoder.read_f64()?,
            decoder.read_f64()?,
            decoder.read_f64()?,
        )?;
        Ok(Self { callback_id, event })
    }
}

#[cfg(target_arch = "wasm32")]
impl crate::StableEncode for CursorMotionDelivery {
    fn encode(&self, encoder: &mut crate::Encoder) -> SjmlResult<()> {
        encoder.write_u32(CLIENT_INPUT_SCHEMA_VERSION)?;
        encoder.write_u64(self.callback_id)?;
        encoder.write_f64(self.event.x)?;
        encoder.write_f64(self.event.y)?;
        encoder.write_f64(self.event.delta_x)?;
        encoder.write_f64(self.event.delta_y)
    }
}

/// Dispatches one raw, observational native cursor-motion callback.
///
/// # Safety
/// The host must pass a live initialized guest allocation and keep it alive until return, then
/// release the input and response with the paired ABI dealloc exports.
#[cfg(target_arch = "wasm32")]
#[export_name = "sjml_v1_client_cursor_motion_dispatch"]
pub unsafe extern "C" fn dispatch_cursor_motion(pointer: u32, length: u32) -> u64 {
    // SAFETY: the host guarantees the input allocation's initialized live span.
    unsafe {
        crate::abi::dispatch_payload(pointer, length, |delivery: CursorMotionDelivery| {
            if DISPATCHING.with(|active| active.replace(true)) {
                return Err(SjmlError::new(
                    ErrorCode::ServiceBusy,
                    "reentrant client input dispatch is forbidden",
                ));
            }
            let result = (|| {
                let callback = CURSOR_MOTION_CALLBACKS
                    .with(|registry| registry.borrow().get(delivery.callback_id))?;
                let mut callback = callback.try_borrow_mut().map_err(|_| {
                    SjmlError::new(
                        ErrorCode::ServiceBusy,
                        "raw cursor-motion callback is already active",
                    )
                })?;
                callback(delivery.event)?;
                Ok(false)
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
        "client input APIs are available only in a wasm32 client guest",
    )
}

#[cfg(test)]
mod tests {
    use super::{
        decode_cursor_position, decode_input_state, is_key_down, is_mouse_button_down,
        ClientCursorMotionEvent, ClientCursorMovedEvent, ClientInputAction, ClientInputEvent,
        CursorPosition, GlobalKeyObservation, MouseScrollEvent, MAX_CURSOR_COORDINATE,
        MAX_CURSOR_MOTION_CALLBACKS_PER_MODULE_PER_TICK, MAX_CURSOR_MOTION_COORDINATE,
        MAX_CURSOR_MOTION_DELTA, MAX_CURSOR_MOTION_SUBSCRIPTIONS,
        MAX_CURSOR_MOTION_SUBSCRIPTIONS_PER_MODULE, MAX_CURSOR_MOVED_DELTA,
        MAX_CURSOR_MOVED_SUBSCRIPTIONS, MAX_CURSOR_MOVED_SUBSCRIPTIONS_PER_MODULE,
        MAX_GLFW_KEY_CODE, MAX_MOUSE_BUTTON,
    };

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

    #[test]
    fn global_observer_projection_contains_only_detached_keyboard_fields() {
        let key = ClientInputEvent::Key {
            key_code: 32,
            scan_code: 57,
            action: ClientInputAction::Press,
            modifiers: 0x05,
        };
        assert_eq!(
            GlobalKeyObservation::from_input(key),
            Some(GlobalKeyObservation {
                key_code: 32,
                scan_code: 57,
                action: ClientInputAction::Press,
                modifiers: 0x05,
            })
        );
        assert_eq!(
            GlobalKeyObservation::from_input(ClientInputEvent::MouseButton {
                button: 0,
                action: ClientInputAction::Press,
                modifiers: 0,
            }),
            None
        );
    }

    #[test]
    fn mouse_scroll_events_validate_both_axes_and_fractional_deltas() {
        let event = MouseScrollEvent::new(-0.25, 1.5).unwrap();
        assert_eq!(event.delta_x, -0.25);
        assert_eq!(event.delta_y, 1.5);
        assert!(MouseScrollEvent::new(f64::NAN, 0.0).is_err());
        assert!(MouseScrollEvent::new(0.0, f64::INFINITY).is_err());
        assert!(MouseScrollEvent::new(64.01, 0.0).is_err());
        assert!(MouseScrollEvent::new(0.0, 0.0).is_err());
    }

    #[test]
    fn cursor_movement_events_validate_position_delta_and_subscription_budget() {
        assert_eq!(MAX_CURSOR_MOVED_SUBSCRIPTIONS, 256);
        assert_eq!(MAX_CURSOR_MOVED_SUBSCRIPTIONS_PER_MODULE, 1);
        let event = ClientCursorMovedEvent::new(320.5, 90.25, -4.5, 2.0).unwrap();
        assert_eq!(event.x, 320.5);
        assert_eq!(event.y, 90.25);
        assert_eq!(event.delta_x, -4.5);
        assert_eq!(event.delta_y, 2.0);

        for (x, y, dx, dy) in [
            (f64::NAN, 0.0, 1.0, 0.0),
            (0.0, f64::INFINITY, 1.0, 0.0),
            (-1.0, 0.0, 1.0, 0.0),
            (MAX_CURSOR_COORDINATE + 1.0, 0.0, 1.0, 0.0),
            (0.0, 0.0, f64::NAN, 0.0),
            (0.0, 0.0, 0.0, 0.0),
            (0.0, 0.0, MAX_CURSOR_MOVED_DELTA + 1.0, 0.0),
        ] {
            assert!(ClientCursorMovedEvent::new(x, y, dx, dy).is_err());
        }
    }

    #[test]
    fn raw_cursor_motion_is_physical_pixel_bounded_and_has_separate_subscription_limits() {
        assert_eq!(MAX_CURSOR_MOTION_SUBSCRIPTIONS, 64);
        assert_eq!(MAX_CURSOR_MOTION_SUBSCRIPTIONS_PER_MODULE, 1);
        assert_eq!(MAX_CURSOR_MOTION_CALLBACKS_PER_MODULE_PER_TICK, 16);
        let event = ClientCursorMotionEvent::new(1920.0, 1080.0, -8.5, 1.25).unwrap();
        assert_eq!(event.x, 1920.0);
        assert_eq!(event.y, 1080.0);
        assert_eq!(event.delta_x, -8.5);
        assert_eq!(event.delta_y, 1.25);

        for (x, y, dx, dy) in [
            (f64::NAN, 0.0, 1.0, 0.0),
            (0.0, f64::INFINITY, 1.0, 0.0),
            (-1.0, 0.0, 1.0, 0.0),
            (MAX_CURSOR_MOTION_COORDINATE + 1.0, 0.0, 1.0, 0.0),
            (0.0, 0.0, f64::NAN, 0.0),
            (0.0, 0.0, f64::INFINITY, 0.0),
            (0.0, 0.0, 0.0, 0.0),
            (0.0, 0.0, MAX_CURSOR_MOTION_DELTA + 1.0, 0.0),
        ] {
            assert!(ClientCursorMotionEvent::new(x, y, dx, dy).is_err());
        }
    }

    #[test]
    fn cursor_position_decodes_gui_coordinates_and_rejects_invalid_host_values() {
        let mut encoded = Vec::new();
        encoded.extend_from_slice(&12.5f64.to_le_bytes());
        encoded.extend_from_slice(&240.25f64.to_le_bytes());
        assert_eq!(
            decode_cursor_position(&encoded).unwrap(),
            CursorPosition { x: 12.5, y: 240.25 }
        );

        for (x, y) in [
            (f64::NAN, 0.0),
            (0.0, f64::INFINITY),
            (-0.1, 1.0),
            (1.0, -0.1),
            (1_000_000.1, 0.0),
            (0.0, 1_000_000.1),
        ] {
            let mut invalid = Vec::new();
            invalid.extend_from_slice(&x.to_le_bytes());
            invalid.extend_from_slice(&y.to_le_bytes());
            assert!(decode_cursor_position(&invalid).is_err());
        }
        assert!(decode_cursor_position(&[0; 15]).is_err());
    }

    #[test]
    fn held_state_queries_enforce_glfw_bounds_and_decode_only_boolean_bytes() {
        assert_eq!(MAX_GLFW_KEY_CODE, 348);
        assert_eq!(MAX_MOUSE_BUTTON, 7);
        assert!(is_key_down(-1).is_err());
        assert!(is_key_down(MAX_GLFW_KEY_CODE + 1).is_err());
        assert!(is_mouse_button_down(MAX_MOUSE_BUTTON + 1).is_err());

        assert!(!decode_input_state(&[0]).unwrap());
        assert!(decode_input_state(&[1]).unwrap());
        assert!(decode_input_state(&[2]).is_err());
        assert!(decode_input_state(&[]).is_err());
    }
}
