//! Open a bounded vanilla chest screen for an online player.
//!
//! Menu calls run synchronously on the owning server thread and require both
//! `entity.read` (for the login-scoped player handle) and `menu.open`. The
//! screen is a snapshot. Optional action slots emit `menu.action` callbacks, but the host rejects
//! every attempt to move or take displayed items.

use crate::abi::PlayerHandle;
use crate::serialization::Encoder;
use crate::{ErrorCode, SjmlError, SjmlResult, StableEncode};

pub const MAX_MENU_ROWS: u8 = 6;
pub const MAX_MENU_SLOTS: usize = 54;
pub const MAX_MENU_TEXT_BYTES: usize = 256;
pub const MAX_MENU_WIRE_BYTES: usize = 16 * 1024;
pub const MAX_MENU_ACTION_ID_BYTES: usize = 64;
const WIRE_VERSION: u32 = 2;

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
}
