//! Open and replace a bounded vanilla chest screen for an online player.
//!
//! Menu calls run synchronously on the owning server thread and require both
//! `entity.read` (for the login-scoped player handle) and `menu.open`. Optional
//! action slots emit `menu.action` callbacks, but the host rejects every attempt
//! to move or take displayed items. An update can replace only a menu currently
//! owned by the calling runtime module; it cannot replace another module's or a
//! vanilla screen. Replacements are limited to eight per server tick per module.

use crate::abi::PlayerHandle;
#[cfg(target_arch = "wasm32")]
use crate::abi::{allocate_host_input, deallocate_host_input};
#[cfg(target_arch = "wasm32")]
use crate::serialization::Decoder;
use crate::serialization::Encoder;
#[cfg(target_arch = "wasm32")]
use crate::StableDecode;
use crate::{ErrorCode, SjmlError, SjmlResult, StableEncode};

pub const MAX_MENU_ROWS: u8 = 6;
pub const MAX_MENU_SLOTS: usize = 54;
pub const MAX_MENU_TEXT_BYTES: usize = 256;
pub const MAX_MENU_WIRE_BYTES: usize = 16 * 1024;
pub const MAX_MENU_ACTION_ID_BYTES: usize = 64;
const WIRE_VERSION: u32 = 2;
pub const MAX_EDITABLE_MENU_INPUT_ITEM_IDS: usize = 32;
pub const MAX_EDITABLE_MENU_TRANSFER_COUNT: u8 = 64;
pub const MAX_EDITABLE_MENU_WIRE_BYTES: usize = 16 * 1024;
const EDITABLE_MENU_WIRE_VERSION: u32 = 1;
#[cfg(target_arch = "wasm32")]
const EDITABLE_MENU_STACK_OUTPUT_BYTES: u32 = 1 + 4 + 4 + MAX_MENU_TEXT_BYTES as u32 + 4;

/// One registry item displayed in a menu slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuItem {
    item_id: String,
    count: u8,
    label: String,
    action_id: Option<String>,
}

impl MenuItem {
    /// Creates a menu item from a canonical Minecraft `namespace:path` ID.
    pub fn new(item_id: impl Into<String>) -> SjmlResult<Self> {
        let item_id = item_id.into();
        validate_item_id(&item_id)?;
        Ok(Self {
            item_id,
            count: 1,
            label: String::new(),
            action_id: None,
        })
    }

    /// Sets the visible stack count; the host also checks the registered item's stack limit.
    pub fn count(mut self, count: u8) -> SjmlResult<Self> {
        if !(1..=99).contains(&count) {
            return Err(invalid_argument("menu item count must be between 1 and 99"));
        }
        self.count = count;
        Ok(self)
    }

    /// Sets a plain display label. Empty text uses the item's registered name.
    pub fn label(mut self, label: impl Into<String>) -> SjmlResult<Self> {
        let label = label.into();
        validate_text(&label, true, "menu item label")?;
        self.label = label;
        Ok(self)
    }

    /// Makes the displayed slot invoke the subscribed `menu.action` callback when left-clicked.
    /// This never transfers the displayed item to the player's inventory.
    pub fn action(mut self, action_id: impl Into<String>) -> SjmlResult<Self> {
        let action_id = action_id.into();
        validate_action_id(&action_id)?;
        self.action_id = Some(action_id);
        Ok(self)
    }

    pub fn item_id(&self) -> &str {
        &self.item_id
    }

    pub const fn item_count(&self) -> u8 {
        self.count
    }

    pub fn display_label(&self) -> &str {
        &self.label
    }

    pub fn action_id(&self) -> Option<&str> {
        self.action_id.as_deref()
    }
}

/// Sparse, bounded definition of a read-only vanilla chest menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuDefinition {
    title: String,
    rows: u8,
    slots: Vec<Option<MenuItem>>,
}

impl MenuDefinition {
    /// Creates a one-to-six-row chest screen with a plain UTF-8 title.
    pub fn new(title: impl Into<String>, rows: u8) -> SjmlResult<Self> {
        let title = title.into();
        validate_text(&title, false, "menu title")?;
        if !(1..=MAX_MENU_ROWS).contains(&rows) {
            return Err(invalid_argument("menu rows must be between 1 and 6"));
        }
        Ok(Self {
            title,
            rows,
            slots: vec![None; usize::from(rows) * 9],
        })
    }

    /// Places one item in a visible slot. A slot may be assigned only once.
    pub fn slot(&mut self, index: usize, item: MenuItem) -> SjmlResult<&mut Self> {
        let target = self
            .slots
            .get_mut(index)
            .ok_or_else(|| invalid_argument("menu slot index is outside the visible rows"))?;
        if target.is_some() {
            return Err(invalid_argument("menu slot indices must be unique"));
        }
        *target = Some(item);
        Ok(self)
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub const fn rows(&self) -> u8 {
        self.rows
    }

    pub fn has_actions(&self) -> bool {
        self.slots
            .iter()
            .flatten()
            .any(|item| item.action_id.is_some())
    }

    pub fn encode_wire(&self) -> SjmlResult<Vec<u8>> {
        let mut encoder = Encoder::new();
        encoder.write_u32(WIRE_VERSION)?;
        encoder.write_u32(u32::from(self.rows))?;
        self.title.as_str().encode(&mut encoder)?;
        let count = self.slots.iter().filter(|item| item.is_some()).count();
        encoder.write_u32(u32::try_from(count).map_err(|_| {
            SjmlError::new(
                ErrorCode::ArithmeticOverflow,
                "menu slot count exceeds its wire field",
            )
        })?)?;
        for (index, item) in self.slots.iter().enumerate() {
            if let Some(item) = item {
                encoder.write_u32(index as u32)?;
                item.item_id.as_str().encode(&mut encoder)?;
                encoder.write_u32(u32::from(item.count))?;
                item.label.as_str().encode(&mut encoder)?;
                item.action_id.clone().encode(&mut encoder)?;
            }
        }
        let encoded = encoder.into_bytes();
        if encoded.len() > MAX_MENU_WIRE_BYTES {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "menu definition exceeds the 16 KiB wire limit",
            ));
        }
        Ok(encoded)
    }

    /// Opens the immutable menu for a live player on the server thread.
    pub fn open(&self, player: PlayerHandle) -> SjmlResult<()> {
        let wire = self.encode_wire()?;
        #[cfg(target_arch = "wasm32")]
        {
            let handle = crate::to_bytes(&player)?;
            // SAFETY: both encoded slices remain live, immutable, and inside this instance's linear memory
            // until the synchronous host import returns.
            let status = unsafe {
                imported_open(
                    handle.as_ptr() as usize as u32 as i32,
                    handle.len() as u32 as i32,
                    wire.as_ptr() as usize as u32 as i32,
                    wire.len() as u32 as i32,
                )
            };
            if status == 0 {
                Ok(())
            } else {
                Err(SjmlError::from_abi_code(
                    status as u32,
                    "host rejected the menu request",
                ))
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (player, wire);
            Err(unsupported_target())
        }
    }

    /// Replaces the title, row count, items, and action slots of this module's currently open menu.
    ///
    /// Returns a permission error when the player has closed the module's menu or another screen is
    /// open. The host creates a fresh container-menu instance, so the replacement is delivered by
    /// Minecraft's normal container synchronization path. This operation never modifies vanilla
    /// containers or menus opened by another runtime module.
    pub fn update(&self, player: PlayerHandle) -> SjmlResult<()> {
        let wire = self.encode_wire()?;
        #[cfg(target_arch = "wasm32")]
        {
            let handle = crate::to_bytes(&player)?;
            // SAFETY: both encoded slices remain live, immutable, and inside this instance's linear memory
            // until the synchronous host import returns.
            let status = unsafe {
                imported_update(
                    handle.as_ptr() as usize as u32 as i32,
                    handle.len() as u32 as i32,
                    wire.as_ptr() as usize as u32 as i32,
                    wire.len() as u32 as i32,
                )
            };
            if status == 0 {
                Ok(())
            } else {
                Err(SjmlError::from_abi_code(
                    status as u32,
                    "host rejected the menu replacement",
                ))
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (player, wire);
            Err(unsupported_target())
        }
    }
}

/// Slot role in an editable container menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditableMenuSlot {
    /// The player can insert and remove only the exact registered item IDs in this allowlist.
    Input {
        index: u8,
        allowed_item_ids: Vec<String>,
    },
    /// The player can take items; only the server-side transfer API can place items here.
    Output { index: u8 },
}

impl EditableMenuSlot {
    /// Declares an input slot limited to one through 32 canonical item IDs.
    pub fn input<I, S>(index: u8, allowed_item_ids: I) -> SjmlResult<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        if usize::from(index) >= MAX_MENU_SLOTS {
            return Err(invalid_argument(
                "editable menu slot index must be between 0 and 53",
            ));
        }
        let mut ids = Vec::new();
        for item_id in allowed_item_ids {
            let item_id = item_id.into();
            validate_item_id(&item_id)?;
            if ids.contains(&item_id) {
                return Err(invalid_argument(
                    "editable menu input item IDs must be unique",
                ));
            }
            if ids.len() == MAX_EDITABLE_MENU_INPUT_ITEM_IDS {
                return Err(invalid_argument(
                    "editable menu input slots allow at most 32 item IDs",
                ));
            }
            ids.push(item_id);
        }
        if ids.is_empty() {
            return Err(invalid_argument(
                "editable menu input slots require at least one allowed item ID",
            ));
        }
        Ok(Self::Input {
            index,
            allowed_item_ids: ids,
        })
    }

    /// Declares a take-only output slot. It starts empty and accepts only explicit item transfers.
    pub fn output(index: u8) -> SjmlResult<Self> {
        if usize::from(index) >= MAX_MENU_SLOTS {
            return Err(invalid_argument(
                "editable menu slot index must be between 0 and 53",
            ));
        }
        Ok(Self::Output { index })
    }

    pub const fn index(&self) -> u8 {
        match self {
            Self::Input { index, .. } | Self::Output { index } => *index,
        }
    }
}

/// Transfer endpoint for an existing player inventory or editable menu stack.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EditableMenuEndpoint {
    /// Main inventory and hotbar slots `0..=35`.
    PlayerInventory(u8),
    /// A declared input or output slot in the active editable menu.
    Menu(u8),
}

impl EditableMenuEndpoint {
    fn wire_parts(self) -> SjmlResult<(u32, u32)> {
        match self {
            Self::PlayerInventory(slot) if slot <= 35 => Ok((0, u32::from(slot))),
            Self::Menu(slot) if usize::from(slot) < MAX_MENU_SLOTS => Ok((1, u32::from(slot))),
            Self::PlayerInventory(_) => Err(invalid_argument(
                "player inventory transfer slots are limited to 0..=35",
            )),
            Self::Menu(_) => Err(invalid_argument(
                "editable menu transfer slots are limited to 0..=53",
            )),
        }
    }
}

/// Immutable slot schema for a temporary, server-authoritative editable menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditableMenuDefinition {
    title: String,
    rows: u8,
    slots: Vec<Option<EditableMenuSlot>>,
}

impl EditableMenuDefinition {
    /// Creates an empty one-to-six-row menu. Only declared input/output slots are interactive.
    pub fn new(title: impl Into<String>, rows: u8) -> SjmlResult<Self> {
        let title = title.into();
        validate_text(&title, false, "editable menu title")?;
        if !(1..=MAX_MENU_ROWS).contains(&rows) {
            return Err(invalid_argument(
                "editable menu rows must be between 1 and 6",
            ));
        }
        Ok(Self {
            title,
            rows,
            slots: vec![None; usize::from(rows) * 9],
        })
    }

    /// Assigns one unique slot role within the visible rows.
    pub fn slot(&mut self, slot: EditableMenuSlot) -> SjmlResult<&mut Self> {
        let index = usize::from(slot.index());
        let target = self
            .slots
            .get_mut(index)
            .ok_or_else(|| invalid_argument("editable menu slot is outside the visible rows"))?;
        if target.is_some() {
            return Err(invalid_argument(
                "editable menu slot indices must be unique",
            ));
        }
        *target = Some(slot);
        Ok(self)
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub const fn rows(&self) -> u8 {
        self.rows
    }

    /// Opens this schema for a live player. All slots start empty.
    pub fn open(&self, player: PlayerHandle) -> SjmlResult<()> {
        let wire = self.encode_wire()?;
        #[cfg(target_arch = "wasm32")]
        {
            let handle = crate::to_bytes(&player)?;
            let status = unsafe {
                imported_open_editable(
                    handle.as_ptr() as usize as u32 as i32,
                    handle.len() as u32 as i32,
                    wire.as_ptr() as usize as u32 as i32,
                    wire.len() as u32 as i32,
                )
            };
            status_result(status, "host rejected the editable menu request")
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (player, wire);
            Err(unsupported_target())
        }
    }

    /// Reads a detached item ID/count snapshot from one declared slot in this module's active menu.
    pub fn read_slot(
        &self,
        player: PlayerHandle,
        slot: u8,
    ) -> SjmlResult<Option<EditableMenuStack>> {
        validate_menu_slot(self, slot)?;
        #[cfg(target_arch = "wasm32")]
        {
            let handle = crate::to_bytes(&player)?;
            let snapshot: Option<EditableMenuStack> = read_editable_output(
                EDITABLE_MENU_STACK_OUTPUT_BYTES,
                |output, capacity| unsafe {
                    imported_read_slot(
                        handle.as_ptr() as usize as u32 as i32,
                        handle.len() as u32 as i32,
                        i32::from(slot),
                        output,
                        capacity,
                    )
                },
            )?;
            if let Some(stack) = &snapshot {
                stack.validate()?;
            }
            Ok(snapshot)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (player, slot);
            Err(unsupported_target())
        }
    }

    /// Moves exactly `count` existing items between an inventory slot and a declared menu slot.
    ///
    /// This operation cannot create or transform items. The host preflights item identity,
    /// component equality, allowlists, and capacity before changing either endpoint. Player
    /// clicks may also deposit/withdraw from input slots through vanilla's server-authoritative
    /// container path; output slots are take-only in the client UI.
    pub fn transfer(
        &self,
        player: PlayerHandle,
        source: EditableMenuEndpoint,
        destination: EditableMenuEndpoint,
        count: u8,
    ) -> SjmlResult<()> {
        if count == 0 || count > MAX_EDITABLE_MENU_TRANSFER_COUNT {
            return Err(invalid_argument(
                "editable menu transfer count must be between 1 and 64",
            ));
        }
        let (source_kind, source_slot) = source.wire_parts()?;
        let (destination_kind, destination_slot) = destination.wire_parts()?;
        if source_kind == 0 && destination_kind == 0 {
            return Err(invalid_argument(
                "editable menu transfers cannot move directly between inventory slots",
            ));
        }
        if source_kind == destination_kind && source_slot == destination_slot {
            return Err(invalid_argument(
                "editable menu transfer source and destination must differ",
            ));
        }
        if let EditableMenuEndpoint::Menu(slot) = source {
            validate_menu_slot(self, slot)?;
        }
        if let EditableMenuEndpoint::Menu(slot) = destination {
            validate_menu_slot(self, slot)?;
        }
        #[cfg(target_arch = "wasm32")]
        {
            let handle = crate::to_bytes(&player)?;
            let status = unsafe {
                imported_transfer(
                    handle.as_ptr() as usize as u32 as i32,
                    handle.len() as u32 as i32,
                    source_kind as i32,
                    source_slot as i32,
                    destination_kind as i32,
                    destination_slot as i32,
                    i32::from(count),
                )
            };
            status_result(status, "host rejected the editable menu transfer")
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (
                player,
                source_kind,
                source_slot,
                destination_kind,
                destination_slot,
            );
            Err(unsupported_target())
        }
    }

    pub fn encode_wire(&self) -> SjmlResult<Vec<u8>> {
        let mut encoder = Encoder::new();
        encoder.write_u32(EDITABLE_MENU_WIRE_VERSION)?;
        encoder.write_u32(u32::from(self.rows))?;
        self.title.as_str().encode(&mut encoder)?;
        let count = self.slots.iter().filter(|slot| slot.is_some()).count();
        encoder.write_length(count)?;
        for (index, slot) in self.slots.iter().enumerate() {
            if let Some(slot) = slot {
                encoder.write_length(index)?;
                match slot {
                    EditableMenuSlot::Input {
                        allowed_item_ids, ..
                    } => {
                        encoder.write_u32(0)?;
                        encoder.write_length(allowed_item_ids.len())?;
                        for item_id in allowed_item_ids {
                            item_id.as_str().encode(&mut encoder)?;
                        }
                    }
                    EditableMenuSlot::Output { .. } => {
                        encoder.write_u32(1)?;
                        encoder.write_u32(0)?;
                    }
                }
            }
        }
        let bytes = encoder.into_bytes();
        if bytes.len() > MAX_EDITABLE_MENU_WIRE_BYTES {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "editable menu definition exceeds the 16 KiB wire limit",
            ));
        }
        Ok(bytes)
    }
}

/// Detached item ID and count from one editable menu slot.
#[derive(Debug, Clone, PartialEq, Eq, crate::StableEncode, crate::StableDecode)]
#[sjml(version = 1)]
pub struct EditableMenuStack {
    pub item_id: String,
    pub count: u32,
}

impl EditableMenuStack {
    pub fn validate(&self) -> SjmlResult<()> {
        validate_item_id(&self.item_id)?;
        if !(1..=u32::from(MAX_EDITABLE_MENU_TRANSFER_COUNT)).contains(&self.count) {
            return Err(invalid_argument(
                "editable menu stack count must be between 1 and 64",
            ));
        }
        Ok(())
    }
}

fn validate_menu_slot(menu: &EditableMenuDefinition, slot: u8) -> SjmlResult<()> {
    if usize::from(slot) >= usize::from(menu.rows) * 9 || menu.slots[usize::from(slot)].is_none() {
        return Err(invalid_argument(
            "editable menu operations require a declared input/output slot",
        ));
    }
    Ok(())
}

/// Detached schema-v1 payload for one clicked action slot.
#[derive(Debug, Clone, Eq, PartialEq, crate::StableEncode, crate::StableDecode)]
#[sjml(version = 1)]
pub struct MenuAction {
    /// Canonical lowercase UUID of the online player who clicked.
    pub player_id: String,
    /// Zero-based index in the displayed chest rows.
    pub slot_index: u32,
    /// Action identifier assigned to the slot by the guest.
    pub action_id: String,
}

impl MenuAction {
    /// Validates a detached callback record before user code consumes it.
    pub fn validate(&self) -> SjmlResult<()> {
        if self.player_id.len() != 36
            || !self.player_id.bytes().enumerate().all(|(index, byte)| {
                if [8, 13, 18, 23].contains(&index) {
                    byte == b'-'
                } else {
                    byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)
                }
            })
            || self.slot_index >= MAX_MENU_SLOTS as u32
        {
            return Err(invalid_argument("menu action player or slot is invalid"));
        }
        validate_action_id(&self.action_id)
    }
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_menu_v1")]
extern "C" {
    #[link_name = "open"]
    fn imported_open(
        player_pointer: i32,
        player_length: i32,
        menu_pointer: i32,
        menu_length: i32,
    ) -> i32;
    #[link_name = "update"]
    fn imported_update(
        player_pointer: i32,
        player_length: i32,
        menu_pointer: i32,
        menu_length: i32,
    ) -> i32;
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_editable_menu_v1")]
extern "C" {
    #[link_name = "open"]
    fn imported_open_editable(
        player_pointer: i32,
        player_length: i32,
        menu_pointer: i32,
        menu_length: i32,
    ) -> i32;
    #[link_name = "read_slot"]
    fn imported_read_slot(
        player_pointer: i32,
        player_length: i32,
        slot: i32,
        output_pointer: i32,
        output_capacity: i32,
    ) -> i32;
    #[link_name = "transfer"]
    fn imported_transfer(
        player_pointer: i32,
        player_length: i32,
        source_kind: i32,
        source_slot: i32,
        destination_kind: i32,
        destination_slot: i32,
        count: i32,
    ) -> i32;
}

#[cfg(target_arch = "wasm32")]
fn status_result(status: i32, message: &str) -> SjmlResult<()> {
    if status == 0 {
        Ok(())
    } else {
        Err(SjmlError::from_abi_code(status as u32, message))
    }
}

#[cfg(target_arch = "wasm32")]
fn read_editable_output<T: StableDecode>(
    capacity: u32,
    invoke: impl FnOnce(i32, i32) -> i32,
) -> SjmlResult<T> {
    let pointer = allocate_host_input(capacity);
    if pointer == 0 {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "could not allocate editable menu output buffer",
        ));
    }
    // SAFETY: initialize the complete allocation before the synchronous host write.
    unsafe { core::ptr::write_bytes(pointer as *mut u8, 0, capacity as usize) };
    let status = invoke(pointer as i32, capacity as i32);
    let result = if status == 0 {
        // SAFETY: the buffer remains allocated until stable decoding completes.
        let bytes = unsafe { core::slice::from_raw_parts(pointer as *const u8, capacity as usize) };
        Decoder::new(bytes).and_then(|mut decoder| T::decode(&mut decoder))
    } else {
        Err(SjmlError::from_abi_code(
            status as u32,
            "host rejected the editable menu read",
        ))
    };
    // SAFETY: release the exact allocation after the synchronous call and decode.
    unsafe { deallocate_host_input(pointer, capacity) };
    result
}

fn validate_item_id(value: &str) -> SjmlResult<()> {
    if value.len() > MAX_MENU_TEXT_BYTES {
        return Err(invalid_argument(
            "menu item ID exceeds its UTF-8 byte limit",
        ));
    }
    let Some((namespace, path)) = value.split_once(':') else {
        return Err(invalid_argument(
            "menu item ID must use canonical namespace:path syntax",
        ));
    };
    let namespace_valid = !namespace.is_empty()
        && namespace.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_.-".contains(&byte)
        });
    let path_valid = !path.is_empty()
        && path.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"/._-".contains(&byte)
        });
    if !namespace_valid || !path_valid {
        return Err(invalid_argument(
            "menu item ID must use canonical namespace:path syntax",
        ));
    }
    Ok(())
}

fn validate_action_id(value: &str) -> SjmlResult<()> {
    if value.is_empty()
        || value.len() > MAX_MENU_ACTION_ID_BYTES
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_.-".contains(&byte)
        })
    {
        return Err(invalid_argument(
            "menu action ID must be 1..64 lowercase ASCII characters",
        ));
    }
    Ok(())
}

fn validate_text(value: &str, allow_empty: bool, label: &str) -> SjmlResult<()> {
    if (!allow_empty && value.is_empty())
        || value.len() > MAX_MENU_TEXT_BYTES
        || value.chars().any(char::is_control)
    {
        return Err(invalid_argument(&format!(
            "{label} must be printable UTF-8 within the 256-byte limit"
        )));
    }
    Ok(())
}

fn invalid_argument(message: &str) -> SjmlError {
    SjmlError::new(ErrorCode::InvalidArgument, message)
}

#[cfg(not(target_arch = "wasm32"))]
fn unsupported_target() -> SjmlError {
    SjmlError::new(
        ErrorCode::UnknownErrorCode,
        "menu imports require the wasm32-unknown-unknown target",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_builder_bounds_rows_slots_item_ids_and_text() {
        assert_eq!(
            MenuDefinition::new("rewards", 0).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            MenuDefinition::new("", 1).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            MenuItem::new("../diamond").unwrap_err().code(),
            ErrorCode::InvalidArgument
        );

        let mut menu = MenuDefinition::new("rewards", 1).unwrap();
        let diamond = MenuItem::new("minecraft:diamond").unwrap();
        menu.slot(0, diamond.clone()).unwrap();
        assert_eq!(
            menu.slot(0, diamond).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            menu.slot(9, MenuItem::new("minecraft:stone").unwrap())
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn wire_payload_uses_little_endian_versioned_menu_records() {
        let mut menu = MenuDefinition::new("R", 1).unwrap();
        menu.slot(
            2,
            MenuItem::new("minecraft:diamond")
                .unwrap()
                .count(3)
                .unwrap(),
        )
        .unwrap();
        let encoded = menu.encode_wire().unwrap();
        let expected: &[u8] = &[
            2, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, b'R', 1, 0, 0, 0, 2, 0, 0, 0, 17, 0, 0, 0, b'm',
            b'i', b'n', b'e', b'c', b'r', b'a', b'f', b't', b':', b'd', b'i', b'a', b'm', b'o',
            b'n', b'd', 3, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        assert_eq!(encoded, expected);
    }

    #[test]
    fn action_slot_is_bounded_and_versioned() {
        let mut menu = MenuDefinition::new("actions", 1).unwrap();
        let item = MenuItem::new("minecraft:emerald")
            .unwrap()
            .action("claim")
            .unwrap();
        menu.slot(4, item).unwrap();
        assert!(menu.has_actions());
        assert_eq!(menu.encode_wire().unwrap()[0..4], [2, 0, 0, 0]);
        assert_eq!(
            MenuItem::new("minecraft:stone")
                .unwrap()
                .action("../claim")
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn menu_action_payload_is_schema_versioned_and_rejects_invalid_identity() {
        let action = MenuAction {
            player_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            slot_index: 4,
            action_id: "claim".to_owned(),
        };
        action.validate().unwrap();
        let mut encoder = Encoder::new();
        action.encode(&mut encoder).unwrap();
        let encoded = encoder.into_bytes();
        assert_eq!(encoded[0..4], [1, 0, 0, 0]);
        assert_eq!(u32::from_le_bytes(encoded[44..48].try_into().unwrap()), 4);

        let invalid = MenuAction {
            slot_index: 54,
            ..action
        };
        assert_eq!(
            invalid.validate().unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn native_open_is_reported_as_unsupported() {
        let menu = MenuDefinition::new("rewards", 1).unwrap();
        let player = PlayerHandle::new(1, 1, 1, 1).unwrap();
        assert_eq!(
            menu.open(player).unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn native_update_is_reported_as_unsupported() {
        let menu = MenuDefinition::new("rewards", 1).unwrap();
        let player = PlayerHandle::new(1, 1, 1, 1).unwrap();
        assert_eq!(
            menu.update(player).unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn editable_menu_builder_limits_roles_slots_and_allowlists() {
        assert_eq!(
            EditableMenuDefinition::new("storage", 0)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            EditableMenuSlot::input(0, Vec::<String>::new())
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            EditableMenuSlot::input(0, ["minecraft:stone", "minecraft:stone"])
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            EditableMenuSlot::input(0, ["../stone"]).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );

        let mut menu = EditableMenuDefinition::new("storage", 1).unwrap();
        menu.slot(EditableMenuSlot::input(0, ["minecraft:stone", "minecraft:dirt"]).unwrap())
            .unwrap();
        menu.slot(EditableMenuSlot::output(1).unwrap()).unwrap();
        assert_eq!(menu.title(), "storage");
        assert_eq!(menu.rows(), 1);
        assert_eq!(
            menu.slot(EditableMenuSlot::output(1).unwrap())
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            menu.slot(EditableMenuSlot::output(9).unwrap())
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn editable_menu_wire_schema_is_bounded_little_endian_and_versioned() {
        let mut menu = EditableMenuDefinition::new("S", 1).unwrap();
        menu.slot(EditableMenuSlot::input(0, ["minecraft:stone"]).unwrap())
            .unwrap();
        menu.slot(EditableMenuSlot::output(1).unwrap()).unwrap();
        let encoded = menu.encode_wire().unwrap();
        let expected: &[u8] = &[
            1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, b'S', 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0,
            0, 15, 0, 0, 0, b'm', b'i', b'n', b'e', b'c', b'r', b'a', b'f', b't', b':', b's', b't',
            b'o', b'n', b'e', 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0,
        ];
        assert_eq!(encoded, expected);
        assert!(encoded.len() <= MAX_EDITABLE_MENU_WIRE_BYTES);
    }

    #[test]
    fn editable_menu_stack_snapshot_uses_stable_optional_wire_schema() {
        let stack = EditableMenuStack {
            item_id: "minecraft:stone".to_owned(),
            count: 3,
        };
        stack.validate().unwrap();
        let mut encoder = Encoder::new();
        Some(stack.clone()).encode(&mut encoder).unwrap();
        let encoded = encoder.into_bytes();
        let decoded: Option<EditableMenuStack> = crate::from_bytes(&encoded).unwrap();
        assert_eq!(decoded, Some(stack));
        assert_eq!(
            EditableMenuStack {
                item_id: "bad".to_owned(),
                count: 1
            }
            .validate()
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            EditableMenuStack {
                item_id: "minecraft:stone".to_owned(),
                count: 65
            }
            .validate()
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn editable_menu_transfer_checks_bounds_before_host_access() {
        let menu = EditableMenuDefinition::new("storage", 1).unwrap();
        let player = PlayerHandle::new(1, 1, 1, 1).unwrap();
        assert_eq!(
            menu.transfer(
                player,
                EditableMenuEndpoint::PlayerInventory(0),
                EditableMenuEndpoint::Menu(0),
                0
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            menu.transfer(
                player,
                EditableMenuEndpoint::PlayerInventory(0),
                EditableMenuEndpoint::PlayerInventory(1),
                1
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            menu.read_slot(player, 0).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }
}
