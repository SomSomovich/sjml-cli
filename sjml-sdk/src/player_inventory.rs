//! Bounded reads and single-slot writes for a live player's inventory.
//!
//! `read` requires both `entity.read` and `inventory.read`. Writes require both `entity.read` and
//! `inventory.write`. All operations are available only to logical-server modules, use the owning
//! server thread, and resolve a live login session on every call.

#[cfg(target_arch = "wasm32")]
use crate::abi::{allocate_host_input, deallocate_host_input};
#[cfg(target_arch = "wasm32")]
use crate::entity::status_result;
use crate::inventory::InventorySnapshot;
#[cfg(any(target_arch = "wasm32", test))]
use crate::inventory::{ItemStackDescriptor, MAX_INVENTORY_SLOTS};
#[cfg(target_arch = "wasm32")]
use crate::to_bytes;
#[cfg(any(target_arch = "wasm32", test))]
use crate::{Decoder, StableDecode, StableEncode};
use crate::{ErrorCode, PlayerHandle, SjmlError, SjmlResult};

/// Maximum number of slots returned by one player inventory query.
pub const MAX_PLAYER_INVENTORY_SLOTS: usize = 128;
/// Highest writable Minecraft player inventory index (hotbar through offhand).
pub const MAX_PLAYER_INVENTORY_SLOT_INDEX: u32 = 40;
/// Maximum item count accepted by the host before applying the registered item's and slot's limits.
pub const MAX_PLAYER_INVENTORY_SLOT_COUNT: u32 = 4096;
/// Maximum item count accepted by one player inventory grant request.
pub const MAX_PLAYER_INVENTORY_GIVE_COUNT: u32 = 4096;
/// Maximum UTF-8 item registry ID length accepted by the player inventory writer.
pub const MAX_PLAYER_INVENTORY_ITEM_ID_BYTES: usize = 256;
/// Maximum durability value accepted for a live player inventory item.
pub const MAX_PLAYER_INVENTORY_DAMAGE: u32 = 65_535;
/// Maximum UTF-8 byte length of a plain player-inventory item custom name.
pub const MAX_PLAYER_INVENTORY_CUSTOM_NAME_BYTES: usize = 1_024;
/// Maximum Unicode scalar count of a plain player-inventory item custom name.
pub const MAX_PLAYER_INVENTORY_CUSTOM_NAME_CODE_POINTS: usize = 256;
/// Maximum UTF-8 path length for a module-owned integer data component.
pub const MAX_PLAYER_INVENTORY_COMPONENT_PATH_BYTES: usize = 256;
/// Maximum UTF-8 registry ID length for a vanilla player-stack enchantment operation.
pub const MAX_PLAYER_INVENTORY_ENCHANTMENT_ID_BYTES: usize = 256;
/// Maximum broad ABI enchantment level; the host applies each registry entry's narrower bounds.
pub const MAX_PLAYER_INVENTORY_ENCHANTMENT_LEVEL: u32 = 255;
/// Maximum number of plain-text lore lines on one player inventory stack.
pub const MAX_PLAYER_INVENTORY_LORE_LINES: usize = 8;
/// Maximum UTF-8 byte length of one plain-text lore line.
pub const MAX_PLAYER_INVENTORY_LORE_LINE_BYTES: usize = 256;
/// Maximum Unicode scalar count of one plain-text lore line.
pub const MAX_PLAYER_INVENTORY_LORE_LINE_CODE_POINTS: usize = 256;
/// Maximum stable schema-v1 bytes in one player inventory lore payload.
pub const MAX_PLAYER_INVENTORY_LORE_BYTES: u32 = 2 * 4 + 8 * (4 + 256);

#[cfg(any(target_arch = "wasm32", test))]
const PLAYER_INVENTORY_LORE_SCHEMA: u32 = 1;

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, Clone, Eq, PartialEq, StableEncode)]
#[sjml(version = 1)]
struct PlayerInventoryLorePayload {
    lines: Vec<String>,
}

#[cfg(any(target_arch = "wasm32", test))]
impl StableDecode for PlayerInventoryLorePayload {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        let schema = decoder.read_u32()?;
        if schema != PLAYER_INVENTORY_LORE_SCHEMA {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                "unsupported player inventory lore schema",
            ));
        }
        let count = decoder.read_length(MAX_PLAYER_INVENTORY_LORE_LINES)?;
        let mut lines = Vec::with_capacity(count);
        for _ in 0..count {
            let line = String::decode(decoder)?;
            validate_lore_line(&line)?;
            lines.push(line);
        }
        Ok(Self { lines })
    }
}

/// The only two updates accepted by [`set_slot`]. New stacks use registered items' default components.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PlayerInventorySlotUpdate<'a> {
    /// Clear the selected slot.
    Clear,
    /// Replace the selected slot with the requested number of a registered item.
    Set { item_id: &'a str, count: u32 },
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_inventory_v1")]
extern "C" {
    #[link_name = "read_player"]
    fn imported_read_player(player: i32, player_length: i32, output: i32, capacity: i32) -> i32;

    #[link_name = "set_player_slot"]
    fn imported_set_player_slot(
        player: i32,
        player_length: i32,
        slot: i32,
        item_id: i32,
        item_id_length: i32,
        count: i32,
    ) -> i32;

    #[link_name = "give_player_item"]
    fn imported_give_player_item(
        player: i32,
        player_length: i32,
        item_id: i32,
        item_id_length: i32,
        count: i32,
        inserted_output: i32,
        inserted_capacity: i32,
    ) -> i32;

    #[link_name = "set_player_damage"]
    fn imported_set_player_damage(player: i32, player_length: i32, slot: i32, damage: i32) -> i32;

    #[link_name = "set_player_item_custom_name"]
    fn imported_set_player_item_custom_name(
        player: i32,
        player_length: i32,
        slot: i32,
        name: i32,
        name_length: i32,
    ) -> i32;

    #[link_name = "clear_player_item_custom_name"]
    fn imported_clear_player_item_custom_name(player: i32, player_length: i32, slot: i32) -> i32;

    #[link_name = "read_player_lore"]
    fn imported_read_player_lore(
        player: i32,
        player_length: i32,
        slot: i32,
        output: i32,
        capacity: i32,
    ) -> i32;

    #[link_name = "set_player_lore"]
    fn imported_set_player_lore(
        player: i32,
        player_length: i32,
        slot: i32,
        payload: i32,
        payload_length: i32,
    ) -> i32;

    #[link_name = "clear_player_lore"]
    fn imported_clear_player_lore(player: i32, player_length: i32, slot: i32) -> i32;

    #[link_name = "read_player_enchantment_level"]
    fn imported_read_player_enchantment_level(
        player: i32,
        player_length: i32,
        slot: i32,
        enchantment_id: i32,
        enchantment_id_length: i32,
        output: i32,
        capacity: i32,
    ) -> i32;

    #[link_name = "set_player_enchantment_level"]
    fn imported_set_player_enchantment_level(
        player: i32,
        player_length: i32,
        slot: i32,
        enchantment_id: i32,
        enchantment_id_length: i32,
        level: i32,
    ) -> i32;

    #[link_name = "remove_player_enchantment"]
    fn imported_remove_player_enchantment(
        player: i32,
        player_length: i32,
        slot: i32,
        enchantment_id: i32,
        enchantment_id_length: i32,
    ) -> i32;

    #[link_name = "read_player_integer_component"]
    fn imported_read_player_integer_component(
        player: i32,
        player_length: i32,
        slot: i32,
        component_path: i32,
        component_path_length: i32,
        output: i32,
        capacity: i32,
    ) -> i32;

    #[link_name = "write_player_integer_component"]
    fn imported_write_player_integer_component(
        player: i32,
        player_length: i32,
        slot: i32,
        component_path: i32,
        component_path_length: i32,
        value: i32,
    ) -> i32;

    #[link_name = "read_player_boolean_component"]
    fn imported_read_player_boolean_component(
        player: i32,
        player_length: i32,
        slot: i32,
        component_path: i32,
        component_path_length: i32,
        output: i32,
        capacity: i32,
    ) -> i32;

    #[link_name = "write_player_boolean_component"]
    fn imported_write_player_boolean_component(
        player: i32,
        player_length: i32,
        slot: i32,
        component_path: i32,
        component_path_length: i32,
        value: i32,
    ) -> i32;
}

/// Reads all slots in the current live player's inventory as a bounded detached snapshot.
///
/// Slot indices follow Minecraft's player inventory order: `0..=8` is the hotbar,
/// `9..=35` is the main storage, `36..=39` is armor (feet through head), and `40` is
/// the offhand. Minecraft 1.21.1 normally exposes 41 slots; the ABI allows up to 128
/// for compatible providers. Empty slots are `None`.
/// The host executes this synchronously on the owning server thread and keeps all item
/// objects private. The call requires declared and administrator-granted `entity.read`
/// and `inventory.read` capabilities.
pub fn read(player: PlayerHandle) -> SjmlResult<InventorySnapshot> {
    #[cfg(target_arch = "wasm32")]
    {
        let player = to_bytes(&player)?;
        let snapshot: Option<InventorySnapshot> = read_output(|output, capacity| {
            // SAFETY: the stable handle bytes and allocated output remain live for this
            // synchronous host call; the host does not retain guest memory.
            unsafe {
                imported_read_player(
                    player.as_ptr() as usize as u32 as i32,
                    player.len() as i32,
                    output,
                    capacity,
                )
            }
        })?;
        let snapshot = snapshot.ok_or_else(|| {
            SjmlError::new(
                ErrorCode::InvalidArgument,
                "host returned an absent player inventory snapshot",
            )
        })?;
        validate_inventory(&snapshot)?;
        Ok(snapshot)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = player;
        Err(unsupported_target())
    }
}

/// Reads one bounded integer data component registered by this module from a nonempty player slot.
///
/// `component_path` is local to this package/module and must have been declared with
/// [`crate::registry::register_integer_component`]. Empty slots or stacks without the component
/// return `Ok(None)`. The host validates the stored value against the registered inclusive range.
/// This server-only operation requires declared and granted `entity.read` and `inventory.read`,
/// the owning server thread, and a live player session handle.
pub fn read_integer_component(
    player: PlayerHandle,
    slot: u32,
    component_path: &str,
) -> SjmlResult<Option<i32>> {
    validate_integer_component_access(slot, component_path)?;
    #[cfg(target_arch = "wasm32")]
    {
        let player = to_bytes(&player)?;
        let value = read_output(|output, capacity| {
            // SAFETY: borrowed handle/path and allocated output stay live through the synchronous import.
            unsafe {
                imported_read_player_integer_component(
                    player.as_ptr() as usize as u32 as i32,
                    player.len() as i32,
                    slot as i32,
                    component_path.as_ptr() as usize as u32 as i32,
                    component_path.len() as i32,
                    output,
                    capacity,
                )
            }
        })?;
        Ok(value)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (player, slot, component_path);
        Err(unsupported_target())
    }
}

/// Writes one module-owned integer data component onto a nonempty live player inventory stack.
///
/// The path must have been declared with [`crate::registry::register_integer_component`]. The host
/// enforces the registered inclusive value range, copies the existing stack before updating only
/// this component, preserves its item/count/other components, and synchronizes the active menu.
/// This server-only operation requires declared and granted `entity.read` and `inventory.write`,
/// the owning server thread, and a live player session handle.
pub fn write_integer_component(
    player: PlayerHandle,
    slot: u32,
    component_path: &str,
    value: i32,
) -> SjmlResult<()> {
    validate_integer_component_access(slot, component_path)?;
    #[cfg(target_arch = "wasm32")]
    {
        let player = to_bytes(&player)?;
        // SAFETY: handle and UTF-8 path remain borrowed for this synchronous host call.
        let status = unsafe {
            imported_write_player_integer_component(
                player.as_ptr() as usize as u32 as i32,
                player.len() as i32,
                slot as i32,
                component_path.as_ptr() as usize as u32 as i32,
                component_path.len() as i32,
                value,
            )
        };
        status_result(
            status,
            "host rejected the player inventory integer component write",
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (player, slot, component_path, value);
        Err(unsupported_target())
    }
}

/// Reads one module-owned boolean component from a non-empty live player-inventory stack.
///
/// The path must name a component registered by this module through
/// [`crate::registry::register_boolean_component`]. An empty slot or unset component returns
/// `Ok(None)`. This requires `entity.read` plus `inventory.read` and is available only to
/// logical-server modules.
pub fn read_boolean_component(
    player: PlayerHandle,
    slot: u32,
    component_path: &str,
) -> SjmlResult<Option<bool>> {
    validate_integer_component_access(slot, component_path)?;
    #[cfg(target_arch = "wasm32")]
    {
        let player = to_bytes(&player)?;
        read_output(|output, capacity| {
            // SAFETY: stable handle/path buffers and output memory live through the synchronous call.
            unsafe {
                imported_read_player_boolean_component(
                    player.as_ptr() as usize as u32 as i32,
                    player.len() as i32,
                    slot as i32,
                    component_path.as_ptr() as usize as u32 as i32,
                    component_path.len() as i32,
                    output,
                    capacity,
                )
            }
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (player, slot, component_path);
        Err(unsupported_target())
    }
}

/// Writes a module-owned boolean component on an existing live player-inventory stack.
///
/// The host verifies ownership and copies the existing stack before changing only this component,
/// preserving its item, count, and all other components. This requires `entity.read` plus
/// `inventory.write` and is available only to logical-server modules.
pub fn write_boolean_component(
    player: PlayerHandle,
    slot: u32,
    component_path: &str,
    value: bool,
) -> SjmlResult<()> {
    validate_integer_component_access(slot, component_path)?;
    #[cfg(target_arch = "wasm32")]
    {
        let player = to_bytes(&player)?;
        // SAFETY: handle/path buffers and the scalar value stay live through the synchronous call.
        let status = unsafe {
            imported_write_player_boolean_component(
                player.as_ptr() as usize as u32 as i32,
                player.len() as i32,
                slot as i32,
                component_path.as_ptr() as usize as u32 as i32,
                component_path.len() as i32,
                i32::from(value),
            )
        };
        status_result(
            status,
            "host rejected the player inventory boolean component write",
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (player, slot, component_path, value);
        Err(unsupported_target())
    }
}

fn validate_integer_component_access(slot: u32, component_path: &str) -> SjmlResult<()> {
    if slot > MAX_PLAYER_INVENTORY_SLOT_INDEX {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player inventory slot must be between 0 and 40",
        ));
    }
    if component_path.is_empty()
        || component_path.len() > MAX_PLAYER_INVENTORY_COMPONENT_PATH_BYTES
        || !component_path.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_.-".contains(&byte)
        })
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "integer component path must contain 1..=256 lowercase ASCII letters, digits, '.', '_' or '-'",
        ));
    }
    Ok(())
}

/// Replaces or clears one slot in a live player's inventory.
///
/// Slot indices are `0..=8` hotbar, `9..=35` main storage, `36..=39` armor (feet through head),
/// and `40` offhand. `Clear` sends an empty item ID and zero count. `Set` accepts only a canonical
/// registered item ID and a count from 1 through 4,096; the host additionally checks the actual
/// registered item's maximum and the target slot's limit before making any change. Armor slots
/// accept one item. New stacks use the item's registered default components; guest code cannot
/// supply or change components or NBT. The active player menu is synchronized after the write.
///
/// Requires declared and administrator-granted `entity.read` and `inventory.write`. It runs only
/// in a logical-server context on the owning server thread and fails for stale or foreign handles.
pub fn set_slot(
    player: PlayerHandle,
    slot: u32,
    update: PlayerInventorySlotUpdate<'_>,
) -> SjmlResult<()> {
    validate_slot_update(slot, update)?;
    let (item_id, count) = match update {
        PlayerInventorySlotUpdate::Clear => ("", 0),
        PlayerInventorySlotUpdate::Set { item_id, count } => (item_id, count),
    };
    #[cfg(target_arch = "wasm32")]
    {
        let player = to_bytes(&player)?;
        // SAFETY: handle bytes and borrowed UTF-8 item ID stay live through the synchronous import.
        let status = unsafe {
            imported_set_player_slot(
                player.as_ptr() as usize as u32 as i32,
                player.len() as i32,
                slot as i32,
                item_id.as_ptr() as usize as u32 as i32,
                item_id.len() as i32,
                count as i32,
            )
        };
        status_result(status, "host rejected the player inventory slot update")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (player, item_id, count);
        Err(unsupported_target())
    }
}

/// Gives a bounded count of a registered item's ordinary default stack to a live player's inventory.
///
/// The host fills compatible stacks first, then empty hotbar/main-storage slots (`0..=35`); armor and
/// offhand slots are never used. The item ID must be canonical and registered, and `count` is
/// limited to `1..=4096`. The host creates only the item's default components; the guest cannot
/// supply stack components or NBT. The return value is the number inserted, which can be less than
/// requested when inventory space is limited. The remainder stays with the caller; it is never
/// dropped or silently discarded. Existing stacks merge only when their item and components match.
///
/// Requires declared and administrator-granted `entity.read` and `inventory.write`. This runs only
/// in a logical-server context on the owning server thread and fails for stale or foreign handles.
pub fn give_item(player: PlayerHandle, item_id: &str, count: u32) -> SjmlResult<u32> {
    validate_give_item(item_id, count)?;
    #[cfg(target_arch = "wasm32")]
    {
        let player = to_bytes(&player)?;
        let mut inserted = 0i32;
        // SAFETY: the handle, item ID, and writable four-byte result remain live through the import.
        let status = unsafe {
            imported_give_player_item(
                player.as_ptr() as usize as u32 as i32,
                player.len() as i32,
                item_id.as_ptr() as usize as u32 as i32,
                item_id.len() as i32,
                count as i32,
                (&mut inserted as *mut i32) as usize as u32 as i32,
                core::mem::size_of::<i32>() as i32,
            )
        };
        status_result(status, "host rejected the player inventory item grant")?;
        if inserted < 0 || inserted as u32 > count {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "host returned an invalid inserted item count",
            ));
        }
        Ok(inserted as u32)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (player, item_id, count);
        Err(unsupported_target())
    }
}

fn validate_give_item(item_id: &str, count: u32) -> SjmlResult<()> {
    if !valid_item_id(item_id)
        || item_id == "minecraft:air"
        || count == 0
        || count > MAX_PLAYER_INVENTORY_GIVE_COUNT
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player item grants require a canonical non-air item ID and count in 1..=4096",
        ));
    }
    Ok(())
}

/// Sets the current durability on a nonempty damageable stack in a live player's inventory.
///
/// Slot indices use the same layout as [`read`]. The host preserves all existing item components,
/// rejects nondamageable/empty items and values greater than or equal to the item's maximum damage,
/// then synchronizes the active menu. The requested value is capped at 65,535. Requires declared
/// and administrator-granted `entity.read` and `inventory.write`, and a logical-server main thread.
pub fn set_damage(player: PlayerHandle, slot: u32, damage: u32) -> SjmlResult<()> {
    validate_damage(slot, damage)?;
    #[cfg(target_arch = "wasm32")]
    {
        let player = to_bytes(&player)?;
        // SAFETY: handle bytes remain live for the synchronous import.
        let status = unsafe {
            imported_set_player_damage(
                player.as_ptr() as usize as u32 as i32,
                player.len() as i32,
                slot as i32,
                damage as i32,
            )
        };
        status_result(
            status,
            "host rejected the player inventory durability update",
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (player, slot, damage);
        Err(unsupported_target())
    }
}

/// Sets the plain literal custom name on an existing stack in the live player's inventory.
///
/// This changes only Minecraft's vanilla `CUSTOM_NAME` component. It keeps the existing item,
/// count, durability, and all unrelated components. Names must contain 1..=256 Unicode scalar
/// values, use at most 1,024 UTF-8 bytes, and contain no control characters. The host copies the
/// existing stack, commits the update, marks the inventory changed, and synchronizes the active
/// player menu. Slot indices use the same layout as [`read`].
///
/// Requires declared and administrator-granted `entity.read` and `inventory.write`; it runs only
/// on the logical-server thread and fails if the slot is empty or the player handle is stale.
pub fn set_item_custom_name(player: PlayerHandle, slot: u32, name: &str) -> SjmlResult<()> {
    validate_custom_name(slot, name)?;
    #[cfg(target_arch = "wasm32")]
    {
        let player = to_bytes(&player)?;
        // SAFETY: handle bytes and borrowed UTF-8 name stay live for this synchronous import.
        let status = unsafe {
            imported_set_player_item_custom_name(
                player.as_ptr() as usize as u32 as i32,
                player.len() as i32,
                slot as i32,
                name.as_ptr() as usize as u32 as i32,
                name.len() as i32,
            )
        };
        status_result(
            status,
            "host rejected the player inventory custom-name update",
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (player, slot, name);
        Err(unsupported_target())
    }
}

/// Clears the plain literal custom name from an existing stack in the live player's inventory.
///
/// The host keeps the existing item, count, durability, and all other components, then marks the
/// inventory changed and synchronizes the active player menu. Slot indices use the same layout as
/// [`read`]. Empty slots are rejected.
///
/// Requires declared and administrator-granted `entity.read` and `inventory.write`; it runs only
/// on the logical-server thread and fails if the player handle is stale.
pub fn clear_item_custom_name(player: PlayerHandle, slot: u32) -> SjmlResult<()> {
    validate_custom_name_slot(slot)?;
    #[cfg(target_arch = "wasm32")]
    {
        let player = to_bytes(&player)?;
        // SAFETY: handle bytes remain live for this synchronous import.
        let status = unsafe {
            imported_clear_player_item_custom_name(
                player.as_ptr() as usize as u32 as i32,
                player.len() as i32,
                slot as i32,
            )
        };
        status_result(
            status,
            "host rejected the player inventory custom-name clear",
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (player, slot);
        Err(unsupported_target())
    }
}

/// Reads the detached plain-text lore lines from an existing player inventory stack.
///
/// Components are flattened to literal text. Styles, click/hover events, and JSON structure are
/// never exposed. An empty result means the stack has no lore. The host rejects values outside
/// eight lines of at most 256 UTF-8 bytes each rather than silently truncating existing lore.
/// Slot indices use the same layout as [`read`].
///
/// Requires declared and administrator-granted `entity.read` and `inventory.read`; this operation
/// runs only on the logical-server thread and resolves the live player session for each call.
pub fn read_lore(player: PlayerHandle, slot: u32) -> SjmlResult<Vec<String>> {
    validate_lore_slot(slot)?;
    #[cfg(target_arch = "wasm32")]
    {
        let player = to_bytes(&player)?;
        let response: PlayerInventoryLorePayload = read_output(|output, capacity| {
            // SAFETY: the handle and allocated output remain live through this synchronous import.
            unsafe {
                imported_read_player_lore(
                    player.as_ptr() as usize as u32 as i32,
                    player.len() as i32,
                    slot as i32,
                    output,
                    capacity,
                )
            }
        })?;
        validate_lore_lines(&response.lines, true)?;
        Ok(response.lines)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (player, slot);
        Err(unsupported_target())
    }
}

/// Sets up to eight plain literal lore lines on an existing player inventory stack.
///
/// Each line is limited to 256 UTF-8 bytes and characters and cannot contain control characters.
/// An empty list is rejected; use [`clear_lore`] to remove the component. The host replaces only
/// Minecraft's vanilla lore component, preserving the item, count, damage, name, enchantments,
/// and all other components. Slot indices use the same layout as [`read`].
///
/// Requires declared and administrator-granted `entity.read` and `inventory.write`; this operation
/// runs only on the logical-server thread, synchronizes the active inventory menu, and rejects an
/// empty slot or stale/foreign player handle.
pub fn set_lore(player: PlayerHandle, slot: u32, lines: &[String]) -> SjmlResult<()> {
    validate_lore_slot(slot)?;
    validate_lore_lines(lines, false)?;
    #[cfg(target_arch = "wasm32")]
    {
        let player = to_bytes(&player)?;
        let payload = to_bytes(&PlayerInventoryLorePayload {
            lines: lines.to_vec(),
        })?;
        if payload.len() > MAX_PLAYER_INVENTORY_LORE_BYTES as usize {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "player inventory lore exceeds its 2,088-byte ABI limit",
            ));
        }
        // SAFETY: the handle and encoded payload remain live through this synchronous host call.
        let status = unsafe {
            imported_set_player_lore(
                player.as_ptr() as usize as u32 as i32,
                player.len() as i32,
                slot as i32,
                payload.as_ptr() as usize as u32 as i32,
                payload.len() as i32,
            )
        };
        status_result(status, "host rejected the player inventory lore update")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (player, slot, lines);
        Err(unsupported_target())
    }
}

/// Clears only Minecraft's vanilla lore component from an existing player inventory stack.
///
/// The item, count, damage, name, enchantments, and all other components are preserved. Slot
/// indices use the same layout as [`read`]; empty slots are rejected.
///
/// Requires declared and administrator-granted `entity.read` and `inventory.write`; this operation
/// runs only on the logical-server thread and synchronizes the active inventory menu.
pub fn clear_lore(player: PlayerHandle, slot: u32) -> SjmlResult<()> {
    validate_lore_slot(slot)?;
    #[cfg(target_arch = "wasm32")]
    {
        let player = to_bytes(&player)?;
        // SAFETY: handle bytes remain live for this synchronous host call.
        let status = unsafe {
            imported_clear_player_lore(
                player.as_ptr() as usize as u32 as i32,
                player.len() as i32,
                slot as i32,
            )
        };
        status_result(status, "host rejected the player inventory lore clear")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (player, slot);
        Err(unsupported_target())
    }
}

fn validate_lore_slot(slot: u32) -> SjmlResult<()> {
    if slot > MAX_PLAYER_INVENTORY_SLOT_INDEX {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player inventory slot must be between 0 and 40",
        ));
    }
    Ok(())
}

fn validate_lore_lines(lines: &[String], allow_empty: bool) -> SjmlResult<()> {
    if lines.len() > MAX_PLAYER_INVENTORY_LORE_LINES {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "player inventory lore supports at most eight lines",
        ));
    }
    if !allow_empty && lines.is_empty() {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player inventory lore must contain one to eight lines; use clear_lore to remove it",
        ));
    }
    for line in lines {
        validate_lore_line(line)?;
    }
    Ok(())
}

fn validate_lore_line(line: &str) -> SjmlResult<()> {
    if line.len() > MAX_PLAYER_INVENTORY_LORE_LINE_BYTES
        || line.chars().count() > MAX_PLAYER_INVENTORY_LORE_LINE_CODE_POINTS
    {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "player inventory lore lines are limited to 256 UTF-8 bytes and characters",
        ));
    }
    if line.chars().any(char::is_control) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player inventory lore lines cannot contain control characters",
        ));
    }
    Ok(())
}

/// Reads the level of one registered enchantment on an existing player inventory stack.
///
/// Returns `None` when the item does not currently have that enchantment. The ID must be a
/// canonical registry ID such as `minecraft:sharpness`; the host resolves it from the active
/// world's enchantment registry. Enchanted books read their stored enchantments. The stack must
/// be nonempty. Slot indices use the same layout as [`read`].
///
/// Requires declared and administrator-granted `entity.read` and `inventory.read`; this runs only
/// on the logical-server thread and requires a live player session handle.
pub fn read_enchantment_level(
    player: PlayerHandle,
    slot: u32,
    enchantment_id: &str,
) -> SjmlResult<Option<u32>> {
    validate_enchantment_access(slot, enchantment_id)?;
    #[cfg(target_arch = "wasm32")]
    {
        let player = to_bytes(&player)?;
        let level: Option<u32> = read_output(|output, capacity| {
            // SAFETY: handle/ID and allocated output remain valid for this synchronous host call.
            unsafe {
                imported_read_player_enchantment_level(
                    player.as_ptr() as usize as u32 as i32,
                    player.len() as i32,
                    slot as i32,
                    enchantment_id.as_ptr() as usize as u32 as i32,
                    enchantment_id.len() as i32,
                    output,
                    capacity,
                )
            }
        })?;
        if level.is_some_and(|value| value == 0 || value > MAX_PLAYER_INVENTORY_ENCHANTMENT_LEVEL) {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "host returned an invalid player inventory enchantment level",
            ));
        }
        Ok(level)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (player, slot, enchantment_id);
        Err(unsupported_target())
    }
}

/// Sets one registered vanilla enchantment on an applicable existing player inventory stack.
///
/// The level must be in `1..=255`; the host additionally enforces the enchantment registry's
/// exact minimum and maximum, item applicability, and conflicts with other enchantments. It
/// changes only the requested enchantment, preserving item, count, durability, and all unrelated
/// components/enchantments. Enchanted books update their stored-enchantment component.
///
/// Requires declared and administrator-granted `entity.read` and `inventory.write`; this runs only
/// on the logical-server thread and rejects empty slots and stale player handles.
pub fn set_enchantment_level(
    player: PlayerHandle,
    slot: u32,
    enchantment_id: &str,
    level: u32,
) -> SjmlResult<()> {
    validate_enchantment_access(slot, enchantment_id)?;
    if !(1..=MAX_PLAYER_INVENTORY_ENCHANTMENT_LEVEL).contains(&level) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player inventory enchantment level must be between 1 and 255",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let player = to_bytes(&player)?;
        // SAFETY: handle and borrowed UTF-8 ID remain live for the synchronous host call.
        let status = unsafe {
            imported_set_player_enchantment_level(
                player.as_ptr() as usize as u32 as i32,
                player.len() as i32,
                slot as i32,
                enchantment_id.as_ptr() as usize as u32 as i32,
                enchantment_id.len() as i32,
                level as i32,
            )
        };
        status_result(
            status,
            "host rejected the player inventory enchantment update",
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (player, slot, enchantment_id, level);
        Err(unsupported_target())
    }
}

/// Removes only one registered enchantment from an existing player inventory stack.
///
/// The item, count, durability, other enchantments, and unrelated components remain unchanged.
/// Removing an enchantment that is already absent succeeds without changing the stack. Enchanted
/// books update their stored-enchantment component. Slot indices use the same layout as [`read`].
///
/// Requires declared and administrator-granted `entity.read` and `inventory.write`; this runs only
/// on the logical-server thread and rejects empty slots and stale player handles.
pub fn remove_enchantment(player: PlayerHandle, slot: u32, enchantment_id: &str) -> SjmlResult<()> {
    validate_enchantment_access(slot, enchantment_id)?;
    #[cfg(target_arch = "wasm32")]
    {
        let player = to_bytes(&player)?;
        // SAFETY: handle and borrowed UTF-8 ID remain live for the synchronous host call.
        let status = unsafe {
            imported_remove_player_enchantment(
                player.as_ptr() as usize as u32 as i32,
                player.len() as i32,
                slot as i32,
                enchantment_id.as_ptr() as usize as u32 as i32,
                enchantment_id.len() as i32,
            )
        };
        status_result(
            status,
            "host rejected the player inventory enchantment removal",
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (player, slot, enchantment_id);
        Err(unsupported_target())
    }
}

fn validate_enchantment_access(slot: u32, enchantment_id: &str) -> SjmlResult<()> {
    if slot > MAX_PLAYER_INVENTORY_SLOT_INDEX {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player inventory slot must be between 0 and 40",
        ));
    }
    if !valid_enchantment_id(enchantment_id) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player enchantment ID must be a canonical registry ID of at most 256 bytes",
        ));
    }
    Ok(())
}

fn valid_enchantment_id(id: &str) -> bool {
    if id.is_empty() || id.len() > MAX_PLAYER_INVENTORY_ENCHANTMENT_ID_BYTES {
        return false;
    }
    let Some((namespace, path)) = id.split_once(':') else {
        return false;
    };
    !namespace.is_empty()
        && !path.is_empty()
        && namespace.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_.-".contains(&byte)
        })
        && path.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"/._-".contains(&byte)
        })
}

fn validate_custom_name(slot: u32, name: &str) -> SjmlResult<()> {
    validate_custom_name_slot(slot)?;
    if name.is_empty() {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player inventory custom name must not be empty; use clear_item_custom_name",
        ));
    }
    if name.len() > MAX_PLAYER_INVENTORY_CUSTOM_NAME_BYTES
        || name.chars().count() > MAX_PLAYER_INVENTORY_CUSTOM_NAME_CODE_POINTS
    {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "player inventory custom name exceeds 1,024 UTF-8 bytes or 256 characters",
        ));
    }
    if name.chars().any(char::is_control) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player inventory custom name cannot contain control characters",
        ));
    }
    Ok(())
}

fn validate_custom_name_slot(slot: u32) -> SjmlResult<()> {
    if slot > MAX_PLAYER_INVENTORY_SLOT_INDEX {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player inventory slot must be between 0 and 40",
        ));
    }
    Ok(())
}

fn validate_damage(slot: u32, damage: u32) -> SjmlResult<()> {
    if slot > MAX_PLAYER_INVENTORY_SLOT_INDEX || damage > MAX_PLAYER_INVENTORY_DAMAGE {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player inventory damage requires a slot in 0..=40 and a value in 0..=65535",
        ));
    }
    Ok(())
}

fn validate_slot_update(slot: u32, update: PlayerInventorySlotUpdate<'_>) -> SjmlResult<()> {
    if slot > MAX_PLAYER_INVENTORY_SLOT_INDEX {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player inventory slot must be between 0 and 40",
        ));
    }
    if let PlayerInventorySlotUpdate::Set { item_id, count } = update {
        if count == 0
            || count > MAX_PLAYER_INVENTORY_SLOT_COUNT
            || !valid_item_id(item_id)
            || item_id == "minecraft:air"
        {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "player inventory writes require a canonical non-air item ID and count in 1..=4096",
            ));
        }
    }
    Ok(())
}

fn valid_item_id(item_id: &str) -> bool {
    if item_id.is_empty() || item_id.len() > MAX_PLAYER_INVENTORY_ITEM_ID_BYTES {
        return false;
    }
    let Some((namespace, path)) = item_id.split_once(':') else {
        return false;
    };
    !namespace.is_empty()
        && !path.is_empty()
        && namespace.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_.-".contains(&byte)
        })
        && path.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"/._-".contains(&byte)
        })
}

#[cfg(any(target_arch = "wasm32", test))]
fn validate_inventory(snapshot: &InventorySnapshot) -> SjmlResult<()> {
    if snapshot.slots.len() > MAX_PLAYER_INVENTORY_SLOTS
        || snapshot.slots.len() > MAX_INVENTORY_SLOTS
    {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "player inventory exceeds the 128-slot limit",
        ));
    }
    for stack in snapshot.slots.iter().flatten() {
        if !valid_item_stack(stack) {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "player inventory contains an invalid item ID or stack size",
            ));
        }
    }
    Ok(())
}

#[cfg(any(target_arch = "wasm32", test))]
fn valid_item_stack(stack: &ItemStackDescriptor) -> bool {
    if stack.item_id.len() > 256
        || !(1..=4096).contains(&stack.count)
        || !(1..=4096).contains(&stack.max_stack_size)
        || !(1..=4096).contains(&stack.slot_limit)
        || stack.count > stack.max_stack_size
        || stack.count > stack.slot_limit
    {
        return false;
    }
    valid_item_id(&stack.item_id)
}

#[cfg(target_arch = "wasm32")]
fn read_output<T: StableDecode>(invoke: impl FnOnce(i32, i32) -> i32) -> SjmlResult<T> {
    let capacity = crate::MAX_PAYLOAD_BYTES as u32;
    let pointer = allocate_host_input(capacity);
    if pointer == 0 {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "could not allocate player inventory output buffer",
        ));
    }
    // SAFETY: initialize the full output allocation before the host writes a bounded value.
    unsafe { core::ptr::write_bytes(pointer as *mut u8, 0, capacity as usize) };
    let status = invoke(pointer as i32, capacity as i32);
    let result = if status == 0 {
        // SAFETY: the initialized allocation remains valid through stable decoding.
        let bytes = unsafe { core::slice::from_raw_parts(pointer as *const u8, capacity as usize) };
        Decoder::new(bytes).and_then(|mut decoder| T::decode(&mut decoder))
    } else {
        Err(SjmlError::from_abi_code(
            status as u32,
            "host rejected the player inventory read",
        ))
    };
    // SAFETY: release the exact allocation after import and decode finish.
    unsafe { deallocate_host_input(pointer, capacity) };
    result
}

#[cfg(not(target_arch = "wasm32"))]
fn unsupported_target() -> SjmlError {
    SjmlError::new(
        ErrorCode::UnknownErrorCode,
        "player inventory operations require the wasm32-unknown-unknown target",
    )
}

#[cfg(test)]
mod tests {
    use super::{
        clear_item_custom_name, clear_lore, give_item, read_enchantment_level, read_lore,
        remove_enchantment, set_damage, set_enchantment_level, set_item_custom_name, set_lore,
        set_slot, validate_custom_name, validate_damage, validate_enchantment_access,
        validate_give_item, validate_inventory, validate_lore_line, validate_lore_lines,
        PlayerInventoryLorePayload, PlayerInventorySlotUpdate,
        MAX_PLAYER_INVENTORY_COMPONENT_PATH_BYTES, MAX_PLAYER_INVENTORY_CUSTOM_NAME_BYTES,
        MAX_PLAYER_INVENTORY_CUSTOM_NAME_CODE_POINTS, MAX_PLAYER_INVENTORY_DAMAGE,
        MAX_PLAYER_INVENTORY_ENCHANTMENT_ID_BYTES, MAX_PLAYER_INVENTORY_ENCHANTMENT_LEVEL,
        MAX_PLAYER_INVENTORY_GIVE_COUNT, MAX_PLAYER_INVENTORY_LORE_LINE_BYTES,
        MAX_PLAYER_INVENTORY_LORE_LINE_CODE_POINTS, MAX_PLAYER_INVENTORY_SLOTS,
        MAX_PLAYER_INVENTORY_SLOT_COUNT, MAX_PLAYER_INVENTORY_SLOT_INDEX,
    };
    #[cfg(not(target_arch = "wasm32"))]
    use super::{
        read, read_boolean_component, read_integer_component, write_boolean_component,
        write_integer_component,
    };
    use crate::inventory::{InventorySnapshot, ItemStackDescriptor};
    use crate::ErrorCode;
    #[cfg(not(target_arch = "wasm32"))]
    use crate::PlayerHandle;

    #[test]
    fn player_inventory_snapshot_is_bounded_to_128_slots() {
        let valid = InventorySnapshot {
            slots: vec![None; MAX_PLAYER_INVENTORY_SLOTS],
        };
        assert!(validate_inventory(&valid).is_ok());

        let oversized = InventorySnapshot {
            slots: vec![None; MAX_PLAYER_INVENTORY_SLOTS + 1],
        };
        assert_eq!(
            validate_inventory(&oversized).unwrap_err().code(),
            ErrorCode::PayloadTooLarge
        );
    }

    #[test]
    fn player_slot_values_keep_only_detached_stable_fields() {
        let snapshot = InventorySnapshot {
            slots: vec![Some(ItemStackDescriptor {
                item_id: "minecraft:iron_ingot".to_owned(),
                count: 12,
                max_stack_size: 64,
                slot_limit: 64,
            })],
        };
        assert!(validate_inventory(&snapshot).is_ok());
    }

    #[test]
    fn player_inventory_validation_rejects_noncanonical_items_and_overstacked_values() {
        for stack in [
            ItemStackDescriptor {
                item_id: "Minecraft:stone".to_owned(),
                count: 1,
                max_stack_size: 64,
                slot_limit: 64,
            },
            ItemStackDescriptor {
                item_id: "minecraft:stone".to_owned(),
                count: 65,
                max_stack_size: 64,
                slot_limit: 64,
            },
            ItemStackDescriptor {
                item_id: "minecraft:stone".to_owned(),
                count: 2,
                max_stack_size: 64,
                slot_limit: 1,
            },
        ] {
            let snapshot = InventorySnapshot {
                slots: vec![Some(stack)],
            };
            assert_eq!(
                validate_inventory(&snapshot).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
        }
    }

    #[test]
    fn player_item_grant_rejects_invalid_registry_ids_and_unbounded_counts() {
        for (item_id, count) in [
            ("Minecraft:stone", 1),
            ("minecraft:air", 1),
            ("minecraft:stone", 0),
            ("minecraft:stone", MAX_PLAYER_INVENTORY_GIVE_COUNT + 1),
            ("", 1),
        ] {
            assert_eq!(
                validate_give_item(item_id, count).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
        }
        assert!(validate_give_item("minecraft:diamond", MAX_PLAYER_INVENTORY_GIVE_COUNT).is_ok());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_item_grant_fails_closed_after_local_validation() {
        let player = PlayerHandle::new(1, 2, 3, 4).unwrap();
        assert_eq!(
            give_item(player, "minecraft:diamond", 1)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            give_item(player, "minecraft:air", 1).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_read_fails_closed_after_typed_handle_validation() {
        let player = PlayerHandle::new(1, 2, 3, 4).unwrap();
        assert_eq!(
            read(player).unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn player_slot_writes_validate_slot_clear_and_stack_shape_before_host_access() {
        let player = PlayerHandle::new(1, 2, 3, 4).unwrap();
        for slot in [0, 8, 9, 35, 36, 39, MAX_PLAYER_INVENTORY_SLOT_INDEX] {
            assert_eq!(
                set_slot(player, slot, PlayerInventorySlotUpdate::Clear)
                    .unwrap_err()
                    .code(),
                ErrorCode::UnknownErrorCode,
                "valid slot {slot} reaches the wasm-only host"
            );
        }
        assert_eq!(
            set_slot(player, 41, PlayerInventorySlotUpdate::Clear)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        for (item_id, count) in [
            ("", 1),
            ("Minecraft:stone", 1),
            ("minecraft:air", 1),
            ("minecraft:stone", 0),
            ("minecraft:stone", MAX_PLAYER_INVENTORY_SLOT_COUNT + 1),
        ] {
            assert_eq!(
                set_slot(player, 0, PlayerInventorySlotUpdate::Set { item_id, count },)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument
            );
        }
    }

    #[test]
    fn player_durability_writes_are_bounded_before_host_access() {
        let player = PlayerHandle::new(1, 2, 3, 4).unwrap();
        for (slot, damage) in [
            (0, 0),
            (MAX_PLAYER_INVENTORY_SLOT_INDEX, MAX_PLAYER_INVENTORY_DAMAGE),
        ] {
            assert_eq!(
                set_damage(player, slot, damage).unwrap_err().code(),
                ErrorCode::UnknownErrorCode,
                "valid slot/damage {slot}/{damage} reaches the wasm-only host"
            );
        }
        assert_eq!(
            validate_damage(41, 1).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            validate_damage(0, MAX_PLAYER_INVENTORY_DAMAGE + 1)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            set_damage(player, 0, MAX_PLAYER_INVENTORY_DAMAGE + 1)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn player_inventory_custom_names_are_bounded_plain_text() {
        let player = PlayerHandle::new(1, 2, 3, 4).unwrap();
        assert!(validate_custom_name(40, "Café 🪨").is_ok());
        assert_eq!(
            set_item_custom_name(player, 0, "literal §c name")
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode,
            "a valid name reaches the wasm-only host on native builds"
        );
        assert_eq!(
            clear_item_custom_name(player, 40).unwrap_err().code(),
            ErrorCode::UnknownErrorCode,
            "a valid clear reaches the wasm-only host on native builds"
        );
        for slot in [41, u32::MAX] {
            assert_eq!(
                set_item_custom_name(player, slot, "name")
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument
            );
            assert_eq!(
                clear_item_custom_name(player, slot).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
        }
        for name in ["", "line\nbreak", "tab\tname"] {
            assert_eq!(
                set_item_custom_name(player, 0, name).unwrap_err().code(),
                ErrorCode::InvalidArgument,
                "invalid name {name:?} must be rejected before host access"
            );
        }
        assert_eq!(
            validate_custom_name(
                0,
                &"é".repeat(MAX_PLAYER_INVENTORY_CUSTOM_NAME_BYTES / 2 + 1)
            )
            .unwrap_err()
            .code(),
            ErrorCode::PayloadTooLarge,
            "UTF-8 byte length, not Rust string byte storage details, bounds the name"
        );
        assert_eq!(
            validate_custom_name(
                0,
                &"a".repeat(MAX_PLAYER_INVENTORY_CUSTOM_NAME_CODE_POINTS + 1)
            )
            .unwrap_err()
            .code(),
            ErrorCode::PayloadTooLarge
        );
    }

    #[test]
    fn player_inventory_lore_uses_bounded_literal_schema_v1_lines() {
        let lines = vec![
            "Café 🪨".to_owned(),
            "§cstill literal".to_owned(),
            String::new(),
        ];
        validate_lore_lines(&lines, false).unwrap();
        let encoded = crate::to_bytes(&PlayerInventoryLorePayload {
            lines: lines.clone(),
        })
        .unwrap();
        let expected = [
            1, 0, 0, 0, 3, 0, 0, 0, // schema and count
            10, 0, 0, 0, // first line length
            0x43, 0x61, 0x66, 0xc3, 0xa9, 0x20, 0xf0, 0x9f, 0xaa, 0xa8, 16, 0, 0,
            0, // second line length
            0xc2, 0xa7, 0x63, 0x73, 0x74, 0x69, 0x6c, 0x6c, 0x20, 0x6c, 0x69, 0x74, 0x65, 0x72,
            0x61, 0x6c, 0, 0, 0, 0, // empty third line
        ];
        assert_eq!(expected, encoded.as_slice());
        let decoded: PlayerInventoryLorePayload = crate::from_bytes(&encoded).unwrap();
        assert_eq!(lines, decoded.lines);
        assert!(validate_lore_line(&"é".repeat(MAX_PLAYER_INVENTORY_LORE_LINE_BYTES / 2)).is_ok());
        assert!(validate_lore_line(&"a".repeat(MAX_PLAYER_INVENTORY_LORE_LINE_BYTES)).is_ok());
        assert_eq!(
            read_lore(PlayerHandle::new(1, 2, 3, 4).unwrap(), 0)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode,
            "a valid read reaches the wasm-only host on native builds"
        );
        assert_eq!(
            clear_lore(PlayerHandle::new(1, 2, 3, 4).unwrap(), 0)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode,
            "a valid clear reaches the wasm-only host on native builds"
        );
        assert_eq!(
            set_lore(
                PlayerHandle::new(1, 2, 3, 4).unwrap(),
                0,
                &["literal".to_owned()]
            )
            .unwrap_err()
            .code(),
            ErrorCode::UnknownErrorCode,
            "valid lines reach the wasm-only host on native builds"
        );

        assert_eq!(
            read_lore(PlayerHandle::new(1, 2, 3, 4).unwrap(), 41)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            clear_lore(PlayerHandle::new(1, 2, 3, 4).unwrap(), 41)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            set_lore(PlayerHandle::new(1, 2, 3, 4).unwrap(), 0, &[])
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            set_lore(
                PlayerHandle::new(1, 2, 3, 4).unwrap(),
                0,
                &["x".repeat(257)]
            )
            .unwrap_err()
            .code(),
            ErrorCode::PayloadTooLarge
        );
    }

    #[test]
    fn player_inventory_lore_rejects_control_characters_and_oversized_collections() {
        assert_eq!(
            validate_lore_line("line\nfeed").unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            validate_lore_line(&"é".repeat(MAX_PLAYER_INVENTORY_LORE_LINE_BYTES / 2 + 1))
                .unwrap_err()
                .code(),
            ErrorCode::PayloadTooLarge
        );
        assert_eq!(
            validate_lore_line(&"a".repeat(MAX_PLAYER_INVENTORY_LORE_LINE_CODE_POINTS + 1))
                .unwrap_err()
                .code(),
            ErrorCode::PayloadTooLarge
        );
        assert_eq!(
            validate_lore_lines(&vec!["x".to_owned(); 9], false)
                .unwrap_err()
                .code(),
            ErrorCode::PayloadTooLarge
        );
        assert!(validate_lore_lines(&[], true).is_ok());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn player_inventory_enchantments_validate_registry_ids_slots_and_levels() {
        let player = PlayerHandle::new(1, 2, 3, 4).unwrap();
        for slot in [0, 8, 9, 35, 36, 39, MAX_PLAYER_INVENTORY_SLOT_INDEX] {
            assert!(validate_enchantment_access(slot, "minecraft:sharpness").is_ok());
            assert_eq!(
                read_enchantment_level(player, slot, "minecraft:sharpness")
                    .unwrap_err()
                    .code(),
                ErrorCode::UnknownErrorCode,
                "valid slot {slot} reaches the wasm-only host"
            );
        }
        assert_eq!(
            remove_enchantment(player, 40, "minecraft:sharpness")
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode,
            "valid removal reaches the wasm-only host"
        );
        assert_eq!(
            set_enchantment_level(player, 0, "minecraft:sharpness", 1)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode,
            "valid level reaches the wasm-only host"
        );
        for slot in [41, u32::MAX] {
            assert_eq!(
                read_enchantment_level(player, slot, "minecraft:sharpness")
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument
            );
        }
        for id in [
            "sharpness",
            "Minecraft:sharpness",
            "minecraft:",
            "minecraft:sharpness\n",
        ] {
            assert_eq!(
                read_enchantment_level(player, 0, id).unwrap_err().code(),
                ErrorCode::InvalidArgument,
                "noncanonical registry ID {id:?} must be rejected before host access"
            );
        }
        assert!(validate_enchantment_access(
            0,
            &format!(
                "x:{}",
                "a".repeat(MAX_PLAYER_INVENTORY_ENCHANTMENT_ID_BYTES - 2)
            )
        )
        .is_ok());
        assert_eq!(
            validate_enchantment_access(
                0,
                &format!(
                    "x:{}",
                    "a".repeat(MAX_PLAYER_INVENTORY_ENCHANTMENT_ID_BYTES - 1)
                )
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            set_enchantment_level(player, 0, "minecraft:sharpness", 0)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument,
            "use remove_enchantment instead of a zero level"
        );
        assert_eq!(
            set_enchantment_level(
                player,
                0,
                "minecraft:sharpness",
                MAX_PLAYER_INVENTORY_ENCHANTMENT_LEVEL + 1
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn player_integer_component_paths_and_slots_are_validated_before_host_access() {
        let player = PlayerHandle::new(1, 2, 3, 4).unwrap();
        for slot in [0, 40] {
            assert_eq!(
                read_integer_component(player, slot, "charge")
                    .unwrap_err()
                    .code(),
                ErrorCode::UnknownErrorCode,
                "valid slot/path {slot} reaches the wasm-only host"
            );
            assert_eq!(
                write_integer_component(player, slot, "charge", -7)
                    .unwrap_err()
                    .code(),
                ErrorCode::UnknownErrorCode,
                "valid slot/path {slot} reaches the wasm-only host"
            );
        }
        for slot in [41, u32::MAX] {
            assert_eq!(
                read_integer_component(player, slot, "charge")
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument
            );
            assert_eq!(
                write_integer_component(player, slot, "charge", 1)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument
            );
        }
        for path in ["", "Charge", "../charge", "nested/charge", "café"] {
            assert_eq!(
                read_integer_component(player, 0, path).unwrap_err().code(),
                ErrorCode::InvalidArgument,
                "invalid component path {path:?} must not reach the host"
            );
        }
        let oversized = "a".repeat(MAX_PLAYER_INVENTORY_COMPONENT_PATH_BYTES + 1);
        assert_eq!(
            write_integer_component(player, 0, &oversized, 1)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn player_boolean_component_paths_and_slots_are_validated_before_host_access() {
        let player = PlayerHandle::new(1, 2, 3, 4).unwrap();
        assert_eq!(
            read_boolean_component(player, 0, "enabled")
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            write_boolean_component(player, 40, "enabled", true)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        for slot in [41, u32::MAX] {
            assert_eq!(
                read_boolean_component(player, slot, "enabled")
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument
            );
            assert_eq!(
                write_boolean_component(player, slot, "enabled", true)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument
            );
        }
        let oversized = "x".repeat(257);
        for path in ["", "Enabled", "../enabled", oversized.as_str()] {
            assert_eq!(
                read_boolean_component(player, 0, path).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
            assert_eq!(
                write_boolean_component(player, 0, path, false)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument
            );
        }
    }
}
