//! Bounded item inventory access on loaded blocks. Reads require `world.read` and
//! `inventory.read`; mutations require `world.read` and `inventory.write`.

#[cfg(target_arch = "wasm32")]
use crate::abi::{allocate_host_input, deallocate_host_input};
#[cfg(target_arch = "wasm32")]
use crate::to_bytes;
use crate::{Decoder, ErrorCode, SjmlError, SjmlResult, StableDecode, StableEncode, WorldHandle};

/// Maximum number of slots returned by one block inventory query.
pub const MAX_INVENTORY_SLOTS: usize = 4096;
/// Maximum UTF-8 size for one module-owned string component value.
pub const MAX_STRING_COMPONENT_BYTES: usize = 4096;
/// Maximum UTF-8 size for a module-local string component path.
pub const MAX_STRING_COMPONENT_PATH_BYTES: usize = 256;
/// Maximum plain-text bytes returned for an item's custom name.
pub const MAX_CUSTOM_ITEM_NAME_BYTES: usize = 1024;
/// Maximum Unicode scalar values accepted when setting a custom item name.
pub const MAX_CUSTOM_ITEM_NAME_CHARS: usize = 256;
/// Maximum plain-text lore lines stored on one block inventory stack.
pub const MAX_BLOCK_INVENTORY_LORE_LINES: usize = 8;
/// Maximum UTF-8 byte length of one plain-text block inventory lore line.
pub const MAX_BLOCK_INVENTORY_LORE_LINE_BYTES: usize = 256;
/// Maximum Unicode scalar count of one plain-text block inventory lore line.
pub const MAX_BLOCK_INVENTORY_LORE_LINE_CODE_POINTS: usize = 256;
/// Maximum registry ID bytes accepted by block-inventory enchantment operations.
pub const MAX_BLOCK_INVENTORY_ENCHANTMENT_ID_BYTES: usize = 256;
/// Maximum enchantment level accepted by the block-inventory ABI.
pub const MAX_BLOCK_INVENTORY_ENCHANTMENT_LEVEL: u32 = 255;
/// Maximum stable schema-v1 bytes in one block inventory lore payload.
pub const MAX_BLOCK_INVENTORY_LORE_BYTES: u32 = 2 * 4
    + MAX_BLOCK_INVENTORY_LORE_LINES as u32 * (4 + MAX_BLOCK_INVENTORY_LORE_LINE_BYTES as u32);
/// Maximum UTF-8 bytes returned for one item registry ID.
pub const MAX_ITEM_ID_BYTES: usize = 256;
/// Maximum encoded bytes returned for one optional item stack snapshot.
pub const MAX_ITEM_STACK_SNAPSHOT_BYTES: usize =
    1 + 4 + 4 + MAX_ITEM_ID_BYTES + 3 * 4 + 2 * (1 + 4) + 1 + 4 + MAX_CUSTOM_ITEM_NAME_BYTES;
/// Maximum item count one [`transfer`] call may move.
pub const MAX_ITEM_TRANSFER_COUNT: u32 = 64;
/// Maximum count accepted by the block stack-count setter before item/slot-specific host checks.
pub const MAX_ITEM_STACK_COUNT: u32 = 99;
/// Maximum damage value accepted by the block-inventory durability setter before item validation.
pub const MAX_BLOCK_INVENTORY_DAMAGE: u32 = 65_535;
/// Maximum absolute X/Z coordinate accepted by block-inventory mutations.
pub const MAX_INVENTORY_COORDINATE: i32 = 30_000_000;

/// Integer block coordinates in the world named by a [`WorldHandle`].
#[derive(Debug, Clone, Copy, Eq, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// One of the six sides used for a sided item-handler capability query.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u32)]
pub enum Direction {
    Down = 0,
    Up = 1,
    North = 2,
    South = 3,
    West = 4,
    East = 5,
}

impl StableEncode for Direction {
    fn encode(&self, encoder: &mut crate::Encoder) -> SjmlResult<()> {
        encoder.write_u32(*self as u32)
    }
}

impl StableDecode for Direction {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        match decoder.read_u32()? {
            0 => Ok(Self::Down),
            1 => Ok(Self::Up),
            2 => Ok(Self::North),
            3 => Ok(Self::South),
            4 => Ok(Self::West),
            5 => Ok(Self::East),
            _ => Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "direction discriminant is invalid",
            )),
        }
    }
}

/// Detached stable values for one non-empty inventory slot.
#[derive(Debug, Clone, Eq, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct ItemStackDescriptor {
    /// Canonical lowercase namespace:path item registry ID.
    pub item_id: String,
    /// Current stack count.
    pub count: u32,
    /// Item's current maximum stack size.
    pub max_stack_size: u32,
    /// This handler's limit for the particular slot.
    pub slot_limit: u32,
}

/// Bounded detached values for one non-empty block-inventory slot.
#[derive(Debug, Clone, Eq, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct ItemStackSnapshot {
    /// Canonical lowercase namespace:path item registry ID.
    pub item_id: String,
    /// Current stack count.
    pub count: u32,
    /// Stack-specific maximum count.
    pub max_stack_size: u32,
    /// This handler's limit for the particular slot.
    pub slot_limit: u32,
    /// Current durability damage, when this item is damageable.
    pub damage: Option<u32>,
    /// Maximum durability damage, when this item is damageable.
    pub max_damage: Option<u32>,
    /// Optional normalized plain custom name, with styles and click/hover data removed.
    pub custom_name: Option<String>,
}

impl ItemStackSnapshot {
    /// Validates the detached values and limits returned by the host for one slot.
    pub fn validate(&self) -> SjmlResult<()> {
        if !valid_item_id(&self.item_id) || self.count == 0 || self.max_stack_size == 0 {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "item stack snapshot contains an invalid item ID or size",
            ));
        }
        if self.damage.is_some() != self.max_damage.is_some()
            || self
                .damage
                .zip(self.max_damage)
                .is_some_and(|(damage, maximum)| maximum == 0 || damage > maximum)
        {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "item stack snapshot damage values are inconsistent",
            ));
        }
        if self.custom_name.as_ref().is_some_and(|name| {
            name.len() > MAX_CUSTOM_ITEM_NAME_BYTES || name.chars().count() > 1024
        }) {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "custom item name exceeds its 1024-byte or character limit",
            ));
        }
        Ok(())
    }
}

#[cfg(any(target_arch = "wasm32", test))]
const BLOCK_INVENTORY_LORE_SCHEMA: u32 = 1;

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, Clone, Eq, PartialEq, StableEncode)]
#[sjml(version = 1)]
struct BlockInventoryLorePayload {
    lines: Vec<String>,
}

#[cfg(any(target_arch = "wasm32", test))]
impl StableDecode for BlockInventoryLorePayload {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        let schema = decoder.read_u32()?;
        if schema != BLOCK_INVENTORY_LORE_SCHEMA {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                "unsupported block inventory lore schema",
            ));
        }
        let count = decoder.read_length(MAX_BLOCK_INVENTORY_LORE_LINES)?;
        let mut lines = Vec::with_capacity(count);
        for _ in 0..count {
            let line = String::decode(decoder)?;
            validate_lore_line(&line)?;
            lines.push(line);
        }
        Ok(Self { lines })
    }
}

/// Stable value snapshot of every handler slot. Empty slots are `None`.
#[derive(Debug, Clone, Eq, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct InventorySnapshot {
    pub slots: Vec<Option<ItemStackDescriptor>>,
}

/// A single sided block-handler slot used as an endpoint for [`transfer`].
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct InventoryEndpoint {
    pub position: BlockPos,
    pub side: Option<Direction>,
    pub slot: u32,
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_inventory_v1")]
extern "C" {
    #[link_name = "read"]
    fn imported_read(
        world: i32,
        world_length: i32,
        position: i32,
        position_length: i32,
        side: i32,
        side_length: i32,
        output: i32,
        capacity: i32,
    ) -> i32;

    #[link_name = "read_stack"]
    fn imported_read_stack(
        world: i32,
        world_length: i32,
        position: i32,
        position_length: i32,
        side: i32,
        side_length: i32,
        slot: i32,
        output: i32,
        capacity: i32,
    ) -> i32;

    #[link_name = "read_string_component"]
    fn imported_read_string_component(
        world: i32,
        world_length: i32,
        position: i32,
        position_length: i32,
        side: i32,
        side_length: i32,
        slot: i32,
        component_path: i32,
        component_path_length: i32,
        output: i32,
        capacity: i32,
    ) -> i32;

    #[link_name = "write_string_component"]
    fn imported_write_string_component(
        world: i32,
        world_length: i32,
        position: i32,
        position_length: i32,
        slot: i32,
        component_path: i32,
        component_path_length: i32,
        value: i32,
        value_length: i32,
    ) -> i32;

    #[link_name = "set_custom_name"]
    fn imported_set_custom_name(
        world: i32,
        world_length: i32,
        position: i32,
        position_length: i32,
        slot: i32,
        name: i32,
        name_length: i32,
    ) -> i32;

    #[link_name = "clear_custom_name"]
    fn imported_clear_custom_name(
        world: i32,
        world_length: i32,
        position: i32,
        position_length: i32,
        slot: i32,
    ) -> i32;

    #[link_name = "set_count"]
    fn imported_set_count(
        world: i32,
        world_length: i32,
        position: i32,
        position_length: i32,
        slot: i32,
        count: i32,
    ) -> i32;

    #[link_name = "set_damage"]
    fn imported_set_damage(
        world: i32,
        world_length: i32,
        position: i32,
        position_length: i32,
        slot: i32,
        damage: i32,
    ) -> i32;

    #[link_name = "read_lore"]
    fn imported_read_lore(
        world: i32,
        world_length: i32,
        position: i32,
        position_length: i32,
        slot: i32,
        output: i32,
        capacity: i32,
    ) -> i32;

    #[link_name = "set_lore"]
    fn imported_set_lore(
        world: i32,
        world_length: i32,
        position: i32,
        position_length: i32,
        slot: i32,
        payload: i32,
        payload_length: i32,
    ) -> i32;

    #[link_name = "clear_lore"]
    fn imported_clear_lore(
        world: i32,
        world_length: i32,
        position: i32,
        position_length: i32,
        slot: i32,
    ) -> i32;

    #[link_name = "read_block_enchantment_level"]
    fn imported_read_block_enchantment_level(
        world: i32,
        world_length: i32,
        position: i32,
        position_length: i32,
        slot: i32,
        enchantment_id: i32,
        enchantment_id_length: i32,
        output: i32,
        capacity: i32,
    ) -> i32;

    #[link_name = "set_block_enchantment_level"]
    fn imported_set_block_enchantment_level(
        world: i32,
        world_length: i32,
        position: i32,
        position_length: i32,
        slot: i32,
        enchantment_id: i32,
        enchantment_id_length: i32,
        level: i32,
    ) -> i32;

    #[link_name = "remove_block_enchantment"]
    fn imported_remove_block_enchantment(
        world: i32,
        world_length: i32,
        position: i32,
        position_length: i32,
        slot: i32,
        enchantment_id: i32,
        enchantment_id_length: i32,
    ) -> i32;

    #[link_name = "read_integer_component"]
    fn imported_read_integer_component(
        world: i32,
        world_length: i32,
        position: i32,
        position_length: i32,
        side: i32,
        side_length: i32,
        slot: i32,
        component_path: i32,
        component_path_length: i32,
        output: i32,
        capacity: i32,
    ) -> i32;

    #[link_name = "write_integer_component"]
    fn imported_write_integer_component(
        world: i32,
        world_length: i32,
        position: i32,
        position_length: i32,
        slot: i32,
        component_path: i32,
        component_path_length: i32,
        value: i32,
    ) -> i32;

    #[link_name = "read_float_component"]
    fn imported_read_float_component(
        world: i32,
        world_length: i32,
        position: i32,
        position_length: i32,
        side: i32,
        side_length: i32,
        slot: i32,
        component_path: i32,
        component_path_length: i32,
        output: i32,
        capacity: i32,
    ) -> i32;

    #[link_name = "write_float_component"]
    fn imported_write_float_component(
        world: i32,
        world_length: i32,
        position: i32,
        position_length: i32,
        slot: i32,
        component_path: i32,
        component_path_length: i32,
        value_bits: i32,
    ) -> i32;

    #[link_name = "read_boolean_component"]
    fn imported_read_boolean_component(
        world: i32,
        world_length: i32,
        position: i32,
        position_length: i32,
        side: i32,
        side_length: i32,
        slot: i32,
        component_path: i32,
        component_path_length: i32,
        output: i32,
        capacity: i32,
    ) -> i32;

    #[link_name = "write_boolean_component"]
    fn imported_write_boolean_component(
        world: i32,
        world_length: i32,
        position: i32,
        position_length: i32,
        slot: i32,
        component_path: i32,
        component_path_length: i32,
        value: i32,
    ) -> i32;

    #[link_name = "transfer"]
    fn imported_transfer(
        world: i32,
        world_length: i32,
        source_position: i32,
        source_position_length: i32,
        source_side: i32,
        source_side_length: i32,
        source_slot: i32,
        destination_position: i32,
        destination_position_length: i32,
        destination_side: i32,
        destination_side_length: i32,
        destination_slot: i32,
        count: i32,
        output: i32,
        capacity: i32,
    ) -> i32;

    #[link_name = "move_stack"]
    fn imported_move_stack(
        world: i32,
        world_length: i32,
        source_position: i32,
        source_position_length: i32,
        source_slot: i32,
        destination_position: i32,
        destination_position_length: i32,
        destination_slot: i32,
        count: i32,
    ) -> i32;
}

/// Moves at most `count` items from one slot to another loaded block inventory.
///
/// This is a single host-side transfer, rather than separate public extraction and insertion
/// operations: item values cannot be copied out and inserted again to duplicate them. Both
/// inventories are queried through their sided item-handler capabilities; `None` selects the
/// unsided handler. The operation requires `world.read` and `inventory.write`, runs synchronously
/// on the owning server thread, and never loads chunks. It returns the number of items moved,
/// or zero when either inventory is absent or the destination cannot accept items.
pub fn transfer(
    world: WorldHandle,
    source: InventoryEndpoint,
    destination: InventoryEndpoint,
    count: u32,
) -> SjmlResult<u32> {
    validate_slot(source.slot)?;
    validate_slot(destination.slot)?;
    validate_transfer_count(count)?;
    #[cfg(target_arch = "wasm32")]
    {
        let world = to_bytes(&world)?;
        let source_position = to_bytes(&source.position)?;
        let source_side = to_bytes(&source.side)?;
        let destination_position = to_bytes(&destination.position)?;
        let destination_side = to_bytes(&destination.side)?;
        read_output(|output, capacity| {
            // SAFETY: stable input buffers and the allocated output remain live for this
            // synchronous host call. The host never retains guest memory.
            unsafe {
                imported_transfer(
                    world.as_ptr() as usize as u32 as i32,
                    world.len() as i32,
                    source_position.as_ptr() as usize as u32 as i32,
                    source_position.len() as i32,
                    source_side.as_ptr() as usize as u32 as i32,
                    source_side.len() as i32,
                    source.slot as i32,
                    destination_position.as_ptr() as usize as u32 as i32,
                    destination_position.len() as i32,
                    destination_side.as_ptr() as usize as u32 as i32,
                    destination_side.len() as i32,
                    destination.slot as i32,
                    count as i32,
                    output,
                    capacity,
                )
            }
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (world, source, destination);
        Err(unsupported_target())
    }
}

/// Moves exactly `count` items between two slots in already-loaded block containers.
///
/// The source must contain the full requested count. The destination must be empty or contain
/// the same item with exactly matching components, and the resulting stack must fit both its
/// container and item limits. The operation requires `world.read` and `inventory.write`, runs on
/// the owning server thread, never loads chunks, and changes only the two slot counts.
pub fn move_stack(
    world: WorldHandle,
    source_position: BlockPos,
    source_slot: u32,
    destination_position: BlockPos,
    destination_slot: u32,
    count: u32,
) -> SjmlResult<()> {
    validate_slot(source_slot)?;
    validate_slot(destination_slot)?;
    validate_transfer_count(count)?;
    validate_inventory_position(source_position)?;
    validate_inventory_position(destination_position)?;
    if source_position == destination_position && source_slot == destination_slot {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "inventory move source and destination slots must differ",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let world = to_bytes(&world)?;
        let source_position = to_bytes(&source_position)?;
        let destination_position = to_bytes(&destination_position)?;
        // SAFETY: the stable input buffers remain live for this synchronous host call.
        let status = unsafe {
            imported_move_stack(
                world.as_ptr() as usize as u32 as i32,
                world.len() as i32,
                source_position.as_ptr() as usize as u32 as i32,
                source_position.len() as i32,
                source_slot as i32,
                destination_position.as_ptr() as usize as u32 as i32,
                destination_position.len() as i32,
                destination_slot as i32,
                count as i32,
            )
        };
        status_result(status, "host rejected the block inventory move")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (
            world,
            source_position,
            source_slot,
            destination_position,
            destination_slot,
            count,
        );
        Err(unsupported_target())
    }
}

/// Changes the count of an existing stack in an already-loaded block container.
///
/// The item and every component are preserved. The count must be in `1..=99` and fit both the
/// item's current maximum stack size and the concrete slot's limit. The host requires a
/// modifiable item-handler capability and a resolved loot container; if the handler rejects a
/// write, the host attempts to restore and verifies the original stack. This requires
/// `world.read` and `inventory.write`, runs synchronously on the owning server thread, and never
/// loads chunks.
pub fn set_count(world: WorldHandle, position: BlockPos, slot: u32, count: u32) -> SjmlResult<()> {
    validate_slot(slot)?;
    validate_inventory_position(position)?;
    validate_item_stack_count(count)?;
    #[cfg(target_arch = "wasm32")]
    {
        let world = to_bytes(&world)?;
        let position = to_bytes(&position)?;
        // SAFETY: stable input buffers remain live for this synchronous host call.
        let status = unsafe {
            imported_set_count(
                world.as_ptr() as usize as u32 as i32,
                world.len() as i32,
                position.as_ptr() as usize as u32 as i32,
                position.len() as i32,
                slot as i32,
                count as i32,
            )
        };
        status_result(status, "host rejected the block inventory count update")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (world, position, slot, count);
        Err(unsupported_target())
    }
}

/// Sets durability on an existing damageable stack in an already-loaded block container.
///
/// This changes only the damage value, preserving item identity, count, and all other
/// components. Empty or nondamageable slots are rejected, as are values at or above the item's
/// maximum durability. The input is bounded to `0..=65_535`; the host checks the actual item
/// maximum. This requires `world.read` and `inventory.write`, runs on the owning server thread,
/// and never loads chunks.
pub fn set_damage(
    world: WorldHandle,
    position: BlockPos,
    slot: u32,
    damage: u32,
) -> SjmlResult<()> {
    validate_slot(slot)?;
    validate_inventory_position(position)?;
    validate_block_inventory_damage(damage)?;
    #[cfg(target_arch = "wasm32")]
    {
        let world = to_bytes(&world)?;
        let position = to_bytes(&position)?;
        // SAFETY: stable input buffers remain live for this synchronous host call.
        let status = unsafe {
            imported_set_damage(
                world.as_ptr() as usize as u32 as i32,
                world.len() as i32,
                position.as_ptr() as usize as u32 as i32,
                position.len() as i32,
                slot as i32,
                damage as i32,
            )
        };
        status_result(
            status,
            "host rejected the block inventory durability update",
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (world, position, slot, damage);
        Err(unsupported_target())
    }
}

/// Reads the block's item-handler capability without loading its chunk. `None` requests the
/// unsided capability; a direction performs a normal sided query. An absent capability or
/// unloaded position returns `Ok(None)`. A present zero-slot handler returns an empty snapshot.
pub fn read(
    world: WorldHandle,
    position: BlockPos,
    side: Option<Direction>,
) -> SjmlResult<Option<InventorySnapshot>> {
    #[cfg(target_arch = "wasm32")]
    {
        let world = to_bytes(&world)?;
        let position = to_bytes(&position)?;
        let side = to_bytes(&side)?;
        let snapshot: Option<InventorySnapshot> = read_output(|output, capacity| {
            // SAFETY: all three stable input buffers and the allocated destination remain live
            // for this synchronous host call.
            unsafe {
                imported_read(
                    world.as_ptr() as usize as u32 as i32,
                    world.len() as i32,
                    position.as_ptr() as usize as u32 as i32,
                    position.len() as i32,
                    side.as_ptr() as usize as u32 as i32,
                    side.len() as i32,
                    output,
                    capacity,
                )
            }
        })?;
        if snapshot
            .as_ref()
            .is_some_and(|value| value.slots.len() > MAX_INVENTORY_SLOTS)
        {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "inventory snapshot exceeds the 4096-slot limit",
            ));
        }
        Ok(snapshot)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (world, position, side);
        Err(unsupported_target())
    }
}

/// Reads one detached item stack from the block's loaded item-handler capability.
///
/// Both `world.read` and `inventory.read` are required. `None` selects the
/// unsided handler; a direction performs the normal sided capability query.
/// An absent capability, unloaded position, or empty slot returns `Ok(None)`.
/// The host returns only item ID, bounded stack/durability values, and a plain
/// custom name; it never exposes NBT or arbitrary data components. Chunks are
/// never loaded, and the operation runs synchronously on the server thread.
pub fn read_stack(
    world: WorldHandle,
    position: BlockPos,
    side: Option<Direction>,
    slot: u32,
) -> SjmlResult<Option<ItemStackSnapshot>> {
    validate_slot(slot)?;
    #[cfg(target_arch = "wasm32")]
    {
        let world = to_bytes(&world)?;
        let position = to_bytes(&position)?;
        let side = to_bytes(&side)?;
        let snapshot: Option<ItemStackSnapshot> =
            read_output_with_capacity(MAX_ITEM_STACK_SNAPSHOT_BYTES as u32, |output, capacity| {
                // SAFETY: stable input buffers and output allocation remain live through this call.
                unsafe {
                    imported_read_stack(
                        world.as_ptr() as usize as u32 as i32,
                        world.len() as i32,
                        position.as_ptr() as usize as u32 as i32,
                        position.len() as i32,
                        side.as_ptr() as usize as u32 as i32,
                        side.len() as i32,
                        slot as i32,
                        output,
                        capacity,
                    )
                }
            })?;
        if let Some(snapshot) = &snapshot {
            snapshot.validate()?;
        }
        Ok(snapshot)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (world, position, side);
        Err(unsupported_target())
    }
}

/// Reads one string component declared by this module from a non-empty loaded inventory slot.
/// An absent inventory, empty slot, or unset component returns `Ok(None)`. The supplied path is
/// module-local, like the path passed to [`crate::registry::register_string_component`].
pub fn read_string_component(
    world: WorldHandle,
    position: BlockPos,
    side: Option<Direction>,
    slot: u32,
    component_path: &str,
) -> SjmlResult<Option<String>> {
    validate_component_path(component_path)?;
    validate_slot(slot)?;
    #[cfg(target_arch = "wasm32")]
    {
        let world = to_bytes(&world)?;
        let position = to_bytes(&position)?;
        let side = to_bytes(&side)?;
        let value: Option<String> = read_output(|output, capacity| {
            // SAFETY: stable input buffers and the destination remain live during this synchronous call.
            unsafe {
                imported_read_string_component(
                    world.as_ptr() as usize as u32 as i32,
                    world.len() as i32,
                    position.as_ptr() as usize as u32 as i32,
                    position.len() as i32,
                    side.as_ptr() as usize as u32 as i32,
                    side.len() as i32,
                    slot as i32,
                    component_path.as_ptr() as usize as u32 as i32,
                    component_path.len() as i32,
                    output,
                    capacity,
                )
            }
        })?;
        if value.as_ref().is_some_and(|text| {
            text.len() > MAX_STRING_COMPONENT_BYTES || text.chars().count() > 1024
        }) {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "string component exceeds the 4096-byte or 1024-character ABI limit",
            ));
        }
        Ok(value)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (world, position, side, component_path);
        Err(unsupported_target())
    }
}

/// Reads one module-owned bounded integer component from a loaded, non-empty block-inventory slot.
///
/// The path must name a component registered by this module through
/// [`crate::registry::register_integer_component`]. `None` is returned for an absent inventory,
/// empty slot, or unset component. The optional side selects the block's item-handler capability;
/// the call requires `world.read` and `inventory.read` and runs on the owning server thread.
pub fn read_integer_component(
    world: WorldHandle,
    position: BlockPos,
    side: Option<Direction>,
    slot: u32,
    component_path: &str,
) -> SjmlResult<Option<i32>> {
    validate_component_path(component_path)?;
    validate_slot(slot)?;
    #[cfg(target_arch = "wasm32")]
    {
        let world = to_bytes(&world)?;
        let position = to_bytes(&position)?;
        let side = to_bytes(&side)?;
        read_output(|output, capacity| {
            // SAFETY: stable input buffers and the allocated output remain live for this synchronous call.
            unsafe {
                imported_read_integer_component(
                    world.as_ptr() as usize as u32 as i32,
                    world.len() as i32,
                    position.as_ptr() as usize as u32 as i32,
                    position.len() as i32,
                    side.as_ptr() as usize as u32 as i32,
                    side.len() as i32,
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
        let _ = (world, position, side, component_path);
        Err(unsupported_target())
    }
}

/// Replaces one module-owned string component on a non-empty loaded inventory slot.
/// The host copies the original stack before changing the component, preserving its item,
/// count, and unrelated data. The block entity must implement Minecraft's `Container` API;
/// writes are unsided and do not use sided item-handler capabilities.
pub fn write_string_component(
    world: WorldHandle,
    position: BlockPos,
    slot: u32,
    component_path: &str,
    value: &str,
) -> SjmlResult<()> {
    validate_component_path(component_path)?;
    validate_slot(slot)?;
    if value.len() > MAX_STRING_COMPONENT_BYTES || value.chars().count() > 1024 {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "string component exceeds the 4096-byte or 1024-character ABI limit",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let world = to_bytes(&world)?;
        let position = to_bytes(&position)?;
        // SAFETY: all guest input slices stay live for the synchronous host call; the host copies them.
        let status = unsafe {
            imported_write_string_component(
                world.as_ptr() as usize as u32 as i32,
                world.len() as i32,
                position.as_ptr() as usize as u32 as i32,
                position.len() as i32,
                slot as i32,
                component_path.as_ptr() as usize as u32 as i32,
                component_path.len() as i32,
                value.as_ptr() as usize as u32 as i32,
                value.len() as i32,
            )
        };
        status_result(status, "host rejected the item component write")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (world, position, component_path, value);
        Err(unsupported_target())
    }
}

/// Sets a bounded plain custom display name on an existing non-empty block-container slot.
///
/// This changes only Minecraft's `CUSTOM_NAME` component; the host copies the existing stack,
/// preserving its item, count, and other components. The block entity must implement
/// `Container`. The operation requires `world.read` and `inventory.write`, runs on the owning
/// server thread, and never loads a chunk. Names must contain 1–256 non-control Unicode scalar
/// values and at most 1,024 UTF-8 bytes.
pub fn set_custom_name(
    world: WorldHandle,
    position: BlockPos,
    slot: u32,
    name: &str,
) -> SjmlResult<()> {
    validate_slot(slot)?;
    validate_custom_name(name)?;
    #[cfg(target_arch = "wasm32")]
    {
        let world = to_bytes(&world)?;
        let position = to_bytes(&position)?;
        // SAFETY: stable input buffers remain live for this synchronous host call; the host copies them.
        let status = unsafe {
            imported_set_custom_name(
                world.as_ptr() as usize as u32 as i32,
                world.len() as i32,
                position.as_ptr() as usize as u32 as i32,
                position.len() as i32,
                slot as i32,
                name.as_ptr() as usize as u32 as i32,
                name.len() as i32,
            )
        };
        status_result(status, "host rejected the custom item name")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (world, position, slot, name);
        Err(unsupported_target())
    }
}

/// Clears Minecraft's custom display name from an existing non-empty block-container slot.
///
/// Other item components, item identity, and count are preserved. The block entity must
/// implement `Container`; the operation requires `world.read` and `inventory.write`, runs on the
/// owning server thread, and never loads a chunk.
pub fn clear_custom_name(world: WorldHandle, position: BlockPos, slot: u32) -> SjmlResult<()> {
    validate_slot(slot)?;
    #[cfg(target_arch = "wasm32")]
    {
        let world = to_bytes(&world)?;
        let position = to_bytes(&position)?;
        // SAFETY: stable input buffers remain live for the synchronous host call.
        let status = unsafe {
            imported_clear_custom_name(
                world.as_ptr() as usize as u32 as i32,
                world.len() as i32,
                position.as_ptr() as usize as u32 as i32,
                position.len() as i32,
                slot as i32,
            )
        };
        status_result(status, "host rejected the custom item name clear")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (world, position, slot);
        Err(unsupported_target())
    }
}

/// Reads the bounded plain-text lore from an existing block-container stack.
///
/// Literal text is returned with styles and event data removed. An empty vector means the
/// component is absent, or the selected slot is empty. The host reads only loaded block entities
/// implementing Minecraft's `Container`; it never loads a chunk. Lore supports up to eight lines
/// of at most 256 UTF-8 bytes and Unicode scalar values each.
///
/// Requires `world.read` and `inventory.read`, and runs synchronously on the server thread.
pub fn read_lore(world: WorldHandle, position: BlockPos, slot: u32) -> SjmlResult<Vec<String>> {
    validate_slot(slot)?;
    validate_inventory_position(position)?;
    #[cfg(target_arch = "wasm32")]
    {
        let world = to_bytes(&world)?;
        let position = to_bytes(&position)?;
        let response: BlockInventoryLorePayload =
            read_output_with_capacity(MAX_BLOCK_INVENTORY_LORE_BYTES, |output, capacity| {
                // SAFETY: stable input buffers and the output allocation stay live through this call.
                unsafe {
                    imported_read_lore(
                        world.as_ptr() as usize as u32 as i32,
                        world.len() as i32,
                        position.as_ptr() as usize as u32 as i32,
                        position.len() as i32,
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
        let _ = (world, position, slot);
        Err(unsupported_target())
    }
}

/// Sets one to eight plain literal lore lines on an existing block-container stack.
///
/// Each line is limited to 256 UTF-8 bytes and Unicode scalar values and cannot contain control
/// characters. Use [`clear_lore`] to remove lore. The host changes only Minecraft's vanilla lore
/// component and preserves item identity, count, damage, and every other component. Writes are
/// unsided and require a loaded block entity implementing `Container`; chunks are never loaded.
///
/// Requires `world.read` and `inventory.write`, and runs synchronously on the server thread.
pub fn set_lore(
    world: WorldHandle,
    position: BlockPos,
    slot: u32,
    lines: &[String],
) -> SjmlResult<()> {
    validate_slot(slot)?;
    validate_inventory_position(position)?;
    validate_lore_lines(lines, false)?;
    #[cfg(target_arch = "wasm32")]
    {
        let world = to_bytes(&world)?;
        let position = to_bytes(&position)?;
        let payload = to_bytes(&BlockInventoryLorePayload {
            lines: lines.to_vec(),
        })?;
        if payload.len() > MAX_BLOCK_INVENTORY_LORE_BYTES as usize {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "block inventory lore exceeds its 2,088-byte ABI limit",
            ));
        }
        // SAFETY: stable input buffers remain live for this synchronous host call; the host copies them.
        let status = unsafe {
            imported_set_lore(
                world.as_ptr() as usize as u32 as i32,
                world.len() as i32,
                position.as_ptr() as usize as u32 as i32,
                position.len() as i32,
                slot as i32,
                payload.as_ptr() as usize as u32 as i32,
                payload.len() as i32,
            )
        };
        status_result(status, "host rejected the block inventory lore update")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (world, position, slot, lines);
        Err(unsupported_target())
    }
}

/// Clears only Minecraft's vanilla lore component on an existing block-container stack.
///
/// The item's identity, count, damage, and other components are preserved. The host requires a
/// loaded block entity implementing `Container`; chunks are never loaded. Requires `world.read`
/// and `inventory.write` and runs synchronously on the server thread.
pub fn clear_lore(world: WorldHandle, position: BlockPos, slot: u32) -> SjmlResult<()> {
    validate_slot(slot)?;
    validate_inventory_position(position)?;
    #[cfg(target_arch = "wasm32")]
    {
        let world = to_bytes(&world)?;
        let position = to_bytes(&position)?;
        // SAFETY: stable input buffers remain live for this synchronous host call.
        let status = unsafe {
            imported_clear_lore(
                world.as_ptr() as usize as u32 as i32,
                world.len() as i32,
                position.as_ptr() as usize as u32 as i32,
                position.len() as i32,
                slot as i32,
            )
        };
        status_result(status, "host rejected the block inventory lore clear")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (world, position, slot);
        Err(unsupported_target())
    }
}

/// Reads the level of one registered enchantment on an existing block-inventory stack.
///
/// The item must support the registered enchantment, except for enchanted books, which use
/// Minecraft's stored-enchantments component. The host reads only a loaded block entity
/// implementing `Container`, rejects unresolved random-loot containers, and never loads chunks.
/// An absent enchantment returns `None`; an empty slot is an error. Requires `world.read` and
/// `inventory.read`, and runs synchronously on the server thread.
pub fn read_enchantment_level(
    world: WorldHandle,
    position: BlockPos,
    slot: u32,
    enchantment_id: &str,
) -> SjmlResult<Option<u32>> {
    validate_slot(slot)?;
    validate_inventory_position(position)?;
    validate_enchantment_id(enchantment_id)?;
    #[cfg(target_arch = "wasm32")]
    {
        let world = to_bytes(&world)?;
        let position = to_bytes(&position)?;
        let level: Option<u32> = read_output(|output, capacity| {
            // SAFETY: stable input buffers and output storage remain live during this call.
            unsafe {
                imported_read_block_enchantment_level(
                    world.as_ptr() as usize as u32 as i32,
                    world.len() as i32,
                    position.as_ptr() as usize as u32 as i32,
                    position.len() as i32,
                    slot as i32,
                    enchantment_id.as_ptr() as usize as u32 as i32,
                    enchantment_id.len() as i32,
                    output,
                    capacity,
                )
            }
        })?;
        if level.is_some_and(|value| value == 0 || value > MAX_BLOCK_INVENTORY_ENCHANTMENT_LEVEL) {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "host returned an invalid block inventory enchantment level",
            ));
        }
        Ok(level)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (world, position, slot, enchantment_id);
        Err(unsupported_target())
    }
}

/// Sets one registered enchantment on an applicable existing block-inventory stack.
///
/// The host enforces the active enchantment registry's minimum and maximum level, item
/// applicability, and conflicts with other enchantments. It changes only the requested vanilla
/// enchantments component, preserving item identity, count, damage, lore, and unrelated
/// components. Enchanted books use their stored-enchantments component. Requires `world.read`
/// and `inventory.write`; it runs synchronously on the server thread and never loads chunks.
pub fn set_enchantment_level(
    world: WorldHandle,
    position: BlockPos,
    slot: u32,
    enchantment_id: &str,
    level: u32,
) -> SjmlResult<()> {
    validate_slot(slot)?;
    validate_inventory_position(position)?;
    validate_enchantment_id(enchantment_id)?;
    if !(1..=MAX_BLOCK_INVENTORY_ENCHANTMENT_LEVEL).contains(&level) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "block inventory enchantment level must be between 1 and 255",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let world = to_bytes(&world)?;
        let position = to_bytes(&position)?;
        // SAFETY: stable input buffers remain live for this synchronous host call.
        let status = unsafe {
            imported_set_block_enchantment_level(
                world.as_ptr() as usize as u32 as i32,
                world.len() as i32,
                position.as_ptr() as usize as u32 as i32,
                position.len() as i32,
                slot as i32,
                enchantment_id.as_ptr() as usize as u32 as i32,
                enchantment_id.len() as i32,
                level as i32,
            )
        };
        status_result(
            status,
            "host rejected the block inventory enchantment update",
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (world, position, slot, enchantment_id, level);
        Err(unsupported_target())
    }
}

/// Removes one registered enchantment from an existing block-inventory stack.
///
/// Removing an absent enchantment succeeds without changing the stack. Other enchantments and
/// all unrelated item components are preserved. Requires `world.read` and `inventory.write` and
/// runs synchronously on the server thread without loading chunks.
pub fn remove_enchantment(
    world: WorldHandle,
    position: BlockPos,
    slot: u32,
    enchantment_id: &str,
) -> SjmlResult<()> {
    validate_slot(slot)?;
    validate_inventory_position(position)?;
    validate_enchantment_id(enchantment_id)?;
    #[cfg(target_arch = "wasm32")]
    {
        let world = to_bytes(&world)?;
        let position = to_bytes(&position)?;
        // SAFETY: stable input buffers remain live for this synchronous host call.
        let status = unsafe {
            imported_remove_block_enchantment(
                world.as_ptr() as usize as u32 as i32,
                world.len() as i32,
                position.as_ptr() as usize as u32 as i32,
                position.len() as i32,
                slot as i32,
                enchantment_id.as_ptr() as usize as u32 as i32,
                enchantment_id.len() as i32,
            )
        };
        status_result(
            status,
            "host rejected the block inventory enchantment removal",
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (world, position, slot, enchantment_id);
        Err(unsupported_target())
    }
}

fn validate_enchantment_id(enchantment_id: &str) -> SjmlResult<()> {
    if enchantment_id.len() > MAX_BLOCK_INVENTORY_ENCHANTMENT_ID_BYTES {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "block enchantment ID exceeds the 256-byte limit",
        ));
    }
    if !valid_item_id(enchantment_id) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "block enchantment ID must be a canonical registry ID",
        ));
    }
    Ok(())
}

fn validate_lore_lines(lines: &[String], allow_empty: bool) -> SjmlResult<()> {
    if lines.len() > MAX_BLOCK_INVENTORY_LORE_LINES {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "block inventory lore supports at most eight lines",
        ));
    }
    if !allow_empty && lines.is_empty() {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "block inventory lore must contain one to eight lines; use clear_lore to remove it",
        ));
    }
    for line in lines {
        validate_lore_line(line)?;
    }
    Ok(())
}

fn validate_lore_line(line: &str) -> SjmlResult<()> {
    if line.len() > MAX_BLOCK_INVENTORY_LORE_LINE_BYTES
        || line.chars().count() > MAX_BLOCK_INVENTORY_LORE_LINE_CODE_POINTS
    {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "block inventory lore lines are limited to 256 UTF-8 bytes and characters",
        ));
    }
    if line.chars().any(char::is_control) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "block inventory lore lines cannot contain control characters",
        ));
    }
    Ok(())
}

/// Writes one module-owned integer component on an existing, non-empty container slot.
///
/// The component path must have been registered by this module. The host checks the registered
/// inclusive range, copies the stack before changing the value, and preserves the item, count, and
/// unrelated components. This operation is unsided because it updates a block `Container`;
/// it requires `world.read` and `inventory.write` and runs synchronously on the server thread.
pub fn write_integer_component(
    world: WorldHandle,
    position: BlockPos,
    slot: u32,
    component_path: &str,
    value: i32,
) -> SjmlResult<()> {
    validate_component_path(component_path)?;
    validate_slot(slot)?;
    #[cfg(target_arch = "wasm32")]
    {
        let world = to_bytes(&world)?;
        let position = to_bytes(&position)?;
        // SAFETY: stable input buffers and the scalar value remain live through the synchronous call.
        let status = unsafe {
            imported_write_integer_component(
                world.as_ptr() as usize as u32 as i32,
                world.len() as i32,
                position.as_ptr() as usize as u32 as i32,
                position.len() as i32,
                slot as i32,
                component_path.as_ptr() as usize as u32 as i32,
                component_path.len() as i32,
                value,
            )
        };
        status_result(status, "host rejected the integer component write")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (world, position, component_path, value);
        Err(unsupported_target())
    }
}

/// Reads one module-owned boolean component from a loaded block-inventory slot.
///
/// The path must name a component registered by this module through
/// [`crate::registry::register_boolean_component`]. `None` is returned for an absent inventory,
/// empty slot, or unset component. The optional side selects the block's item-handler capability;
/// the call requires `world.read` and `inventory.read` and runs on the owning server thread.
pub fn read_boolean_component(
    world: WorldHandle,
    position: BlockPos,
    side: Option<Direction>,
    slot: u32,
    component_path: &str,
) -> SjmlResult<Option<bool>> {
    validate_component_path(component_path)?;
    validate_slot(slot)?;
    #[cfg(target_arch = "wasm32")]
    {
        let world = to_bytes(&world)?;
        let position = to_bytes(&position)?;
        let side = to_bytes(&side)?;
        read_output(|output, capacity| {
            // SAFETY: stable input buffers and output memory live through the synchronous call.
            unsafe {
                imported_read_boolean_component(
                    world.as_ptr() as usize as u32 as i32,
                    world.len() as i32,
                    position.as_ptr() as usize as u32 as i32,
                    position.len() as i32,
                    side.as_ptr() as usize as u32 as i32,
                    side.len() as i32,
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
        let _ = (world, position, side, component_path);
        Err(unsupported_target())
    }
}

/// Reads one module-owned bounded finite float component from a loaded block-inventory slot.
///
/// The path must name a component registered by this module through
/// [`crate::registry::register_float_component`]. `None` is returned for an absent inventory,
/// empty slot, or unset component. The optional side selects the block's item-handler capability;
/// the call requires `world.read` and `inventory.read` and runs on the owning server thread.
pub fn read_float_component(
    world: WorldHandle,
    position: BlockPos,
    side: Option<Direction>,
    slot: u32,
    component_path: &str,
) -> SjmlResult<Option<f32>> {
    validate_component_path(component_path)?;
    validate_slot(slot)?;
    #[cfg(target_arch = "wasm32")]
    {
        let world = to_bytes(&world)?;
        let position = to_bytes(&position)?;
        let side = to_bytes(&side)?;
        let value: Option<f32> = read_output(|output, capacity| {
            // SAFETY: stable input buffers and the allocated output live through this sync import.
            unsafe {
                imported_read_float_component(
                    world.as_ptr() as usize as u32 as i32,
                    world.len() as i32,
                    position.as_ptr() as usize as u32 as i32,
                    position.len() as i32,
                    side.as_ptr() as usize as u32 as i32,
                    side.len() as i32,
                    slot as i32,
                    component_path.as_ptr() as usize as u32 as i32,
                    component_path.len() as i32,
                    output,
                    capacity,
                )
            }
        })?;
        if value.is_some_and(|value| !value.is_finite()) {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "host returned a non-finite float component value",
            ));
        }
        Ok(value)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (world, position, side, component_path);
        Err(unsupported_target())
    }
}

/// Writes one module-owned bounded finite float component on an existing, non-empty container slot.
///
/// The component path must have been registered by this module. The host checks the registered
/// inclusive range before accessing the inventory stack, then copies the stack and changes only
/// this value. The operation is unsided because it updates a block `Container`; it requires
/// `world.read` and `inventory.write` and runs on the server thread.
pub fn write_float_component(
    world: WorldHandle,
    position: BlockPos,
    slot: u32,
    component_path: &str,
    value: f32,
) -> SjmlResult<()> {
    validate_component_path(component_path)?;
    validate_slot(slot)?;
    if !value.is_finite() {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "float component value must be finite",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let world = to_bytes(&world)?;
        let position = to_bytes(&position)?;
        // SAFETY: input buffers and the scalar bits remain live through this synchronous call.
        let status = unsafe {
            imported_write_float_component(
                world.as_ptr() as usize as u32 as i32,
                world.len() as i32,
                position.as_ptr() as usize as u32 as i32,
                position.len() as i32,
                slot as i32,
                component_path.as_ptr() as usize as u32 as i32,
                component_path.len() as i32,
                value.to_bits() as i32,
            )
        };
        status_result(status, "host rejected the float component write")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (world, position, slot, component_path, value);
        Err(unsupported_target())
    }
}

/// Writes a module-owned boolean component on an existing, non-empty container slot.
///
/// The host checks that the component belongs to this module, copies the stack before changing
/// the value, and preserves the item, count, and unrelated components. The operation is unsided
/// because it updates a block `Container`; it requires `world.read` and `inventory.write` and runs
/// on the owning server thread.
pub fn write_boolean_component(
    world: WorldHandle,
    position: BlockPos,
    slot: u32,
    component_path: &str,
    value: bool,
) -> SjmlResult<()> {
    validate_component_path(component_path)?;
    validate_slot(slot)?;
    #[cfg(target_arch = "wasm32")]
    {
        let world = to_bytes(&world)?;
        let position = to_bytes(&position)?;
        // SAFETY: input buffers and the scalar value remain live during the synchronous call.
        let status = unsafe {
            imported_write_boolean_component(
                world.as_ptr() as usize as u32 as i32,
                world.len() as i32,
                position.as_ptr() as usize as u32 as i32,
                position.len() as i32,
                slot as i32,
                component_path.as_ptr() as usize as u32 as i32,
                component_path.len() as i32,
                i32::from(value),
            )
        };
        status_result(status, "host rejected the boolean component write")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (world, position, slot, component_path, value);
        Err(unsupported_target())
    }
}

fn validate_component_path(path: &str) -> SjmlResult<()> {
    if path.is_empty()
        || path.len() > MAX_STRING_COMPONENT_PATH_BYTES
        || !path.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_.-".contains(&byte)
        })
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "component path must contain 1..=256 lowercase ASCII letters, digits, '.', '_' or '-'",
        ));
    }
    Ok(())
}

fn validate_custom_name(name: &str) -> SjmlResult<()> {
    if name.is_empty() {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "custom item name must not be empty; use clear_custom_name",
        ));
    }
    if name.len() > MAX_CUSTOM_ITEM_NAME_BYTES || name.chars().count() > MAX_CUSTOM_ITEM_NAME_CHARS
    {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "custom item name exceeds its 1,024-byte or 256-character limit",
        ));
    }
    if name.chars().any(char::is_control) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "custom item name cannot contain control characters",
        ));
    }
    Ok(())
}

fn validate_slot(slot: u32) -> SjmlResult<()> {
    if slot as usize >= MAX_INVENTORY_SLOTS {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "inventory slot must be within the 4096-slot ABI limit",
        ));
    }
    Ok(())
}

fn valid_item_id(id: &str) -> bool {
    let Some((namespace, path)) = id.split_once(':') else {
        return false;
    };
    !namespace.is_empty()
        && !path.is_empty()
        && id.len() <= MAX_ITEM_ID_BYTES
        && namespace.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_.-".contains(&byte)
        })
        && path.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_./-".contains(&byte)
        })
}

fn validate_transfer_count(count: u32) -> SjmlResult<()> {
    if count == 0 || count > MAX_ITEM_TRANSFER_COUNT {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "inventory transfer count must be within 1..=64",
        ));
    }
    Ok(())
}

fn validate_item_stack_count(count: u32) -> SjmlResult<()> {
    if count == 0 || count > MAX_ITEM_STACK_COUNT {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "item stack count must be within 1..=99",
        ));
    }
    Ok(())
}

fn validate_block_inventory_damage(damage: u32) -> SjmlResult<()> {
    if damage > MAX_BLOCK_INVENTORY_DAMAGE {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "block inventory damage must be within 0..=65535",
        ));
    }
    Ok(())
}

fn validate_inventory_position(position: BlockPos) -> SjmlResult<()> {
    if i64::from(position.x).abs() > i64::from(MAX_INVENTORY_COORDINATE)
        || i64::from(position.z).abs() > i64::from(MAX_INVENTORY_COORDINATE)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "inventory x/z coordinates must be within ±30000000",
        ));
    }
    Ok(())
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
fn read_output<T: StableDecode>(invoke: impl FnOnce(i32, i32) -> i32) -> SjmlResult<T> {
    read_output_with_capacity(crate::MAX_PAYLOAD_BYTES as u32, invoke)
}

#[cfg(target_arch = "wasm32")]
fn read_output_with_capacity<T: StableDecode>(
    capacity: u32,
    invoke: impl FnOnce(i32, i32) -> i32,
) -> SjmlResult<T> {
    let pointer = allocate_host_input(capacity);
    if pointer == 0 {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "could not allocate inventory output buffer",
        ));
    }
    // SAFETY: initialize the complete output allocation before host access.
    unsafe { core::ptr::write_bytes(pointer as *mut u8, 0, capacity as usize) };
    let status = invoke(pointer as i32, capacity as i32);
    let result = if status == 0 {
        // SAFETY: initialized memory remains live until stable decoding finishes.
        let bytes = unsafe { core::slice::from_raw_parts(pointer as *const u8, capacity as usize) };
        Decoder::new(bytes).and_then(|mut decoder| T::decode(&mut decoder))
    } else {
        Err(SjmlError::from_abi_code(
            status as u32,
            "host rejected the inventory operation",
        ))
    };
    // SAFETY: release exactly the allocation owned above after the host call and decode finish.
    unsafe { deallocate_host_input(pointer, capacity) };
    result
}

#[cfg(not(target_arch = "wasm32"))]
fn unsupported_target() -> SjmlError {
    SjmlError::new(
        ErrorCode::UnknownErrorCode,
        "inventory operations require the wasm32-unknown-unknown target",
    )
}

#[cfg(test)]
mod tests {
    use super::{
        clear_custom_name, clear_lore, move_stack, read_boolean_component, read_enchantment_level,
        read_float_component, read_integer_component, read_lore, remove_enchantment, set_count,
        set_custom_name, set_damage, set_enchantment_level, set_lore, transfer,
        validate_block_inventory_damage, validate_custom_name, validate_enchantment_id,
        validate_item_stack_count, validate_lore_line, validate_lore_lines, validate_slot,
        validate_transfer_count, write_boolean_component, write_float_component,
        write_integer_component, BlockPos, Direction, InventoryEndpoint, ItemStackSnapshot,
        MAX_BLOCK_INVENTORY_DAMAGE, MAX_BLOCK_INVENTORY_ENCHANTMENT_ID_BYTES,
        MAX_BLOCK_INVENTORY_ENCHANTMENT_LEVEL, MAX_BLOCK_INVENTORY_LORE_BYTES,
        MAX_BLOCK_INVENTORY_LORE_LINES, MAX_BLOCK_INVENTORY_LORE_LINE_BYTES,
        MAX_CUSTOM_ITEM_NAME_BYTES, MAX_CUSTOM_ITEM_NAME_CHARS, MAX_INVENTORY_COORDINATE,
        MAX_INVENTORY_SLOTS, MAX_ITEM_STACK_COUNT, MAX_ITEM_TRANSFER_COUNT,
    };
    use crate::{ErrorCode, WorldHandle};

    #[test]
    fn transfer_count_is_bounded_and_nonzero() {
        assert_eq!(validate_transfer_count(1), Ok(()));
        assert_eq!(validate_transfer_count(MAX_ITEM_TRANSFER_COUNT), Ok(()));
        assert_eq!(
            validate_transfer_count(0).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            validate_transfer_count(MAX_ITEM_TRANSFER_COUNT + 1)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn item_stack_count_is_bounded_and_nonzero() {
        assert_eq!(validate_item_stack_count(1), Ok(()));
        assert_eq!(validate_item_stack_count(MAX_ITEM_STACK_COUNT), Ok(()));
        assert_eq!(
            validate_item_stack_count(0).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            validate_item_stack_count(MAX_ITEM_STACK_COUNT + 1)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn set_count_validates_slot_position_and_generic_count_before_host_call() {
        let world = WorldHandle::new(1, 2, 3, 4).unwrap();
        let position = BlockPos { x: 0, y: 64, z: 0 };
        assert_eq!(
            set_count(world, position, 0, 0).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            set_count(world, position, MAX_INVENTORY_SLOTS as u32, 1)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            set_count(
                world,
                BlockPos {
                    x: MAX_INVENTORY_COORDINATE + 1,
                    y: 64,
                    z: 0,
                },
                0,
                1,
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn block_inventory_damage_is_bounded_before_host_access() {
        assert_eq!(validate_block_inventory_damage(0), Ok(()));
        assert_eq!(
            validate_block_inventory_damage(MAX_BLOCK_INVENTORY_DAMAGE),
            Ok(())
        );
        assert_eq!(
            validate_block_inventory_damage(MAX_BLOCK_INVENTORY_DAMAGE + 1)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        let world = WorldHandle::new(1, 2, 3, 4).unwrap();
        assert_eq!(
            set_damage(
                world,
                BlockPos { x: 0, y: 64, z: 0 },
                0,
                MAX_BLOCK_INVENTORY_DAMAGE + 1,
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn transfer_slots_stay_within_the_inventory_abi_bound() {
        assert_eq!(validate_slot(0), Ok(()));
        assert_eq!(validate_slot((MAX_INVENTORY_SLOTS - 1) as u32), Ok(()));
        assert_eq!(
            validate_slot(MAX_INVENTORY_SLOTS as u32)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn item_stack_snapshot_rejects_invalid_identity_damage_and_custom_name_bounds() {
        let snapshot = ItemStackSnapshot {
            item_id: "minecraft:iron_sword".to_owned(),
            count: 1,
            max_stack_size: 1,
            slot_limit: 64,
            damage: Some(3),
            max_damage: Some(250),
            custom_name: Some("Sword".to_owned()),
        };
        assert_eq!(snapshot.validate(), Ok(()));

        let invalid_id = ItemStackSnapshot {
            item_id: "Minecraft:iron_sword".to_owned(),
            ..snapshot.clone()
        };
        assert_eq!(
            invalid_id.validate().unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        let invalid_damage = ItemStackSnapshot {
            damage: Some(251),
            ..snapshot.clone()
        };
        assert_eq!(
            invalid_damage.validate().unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        let unbounded_name = ItemStackSnapshot {
            custom_name: Some("x".repeat(MAX_CUSTOM_ITEM_NAME_BYTES + 1)),
            ..snapshot
        };
        assert_eq!(
            unbounded_name.validate().unwrap_err().code(),
            ErrorCode::PayloadTooLarge
        );
    }

    #[test]
    fn public_transfer_checks_endpoint_bounds_before_host_access() {
        let world = WorldHandle::new(1, 2, 3, 4).unwrap();
        let endpoint = InventoryEndpoint {
            position: BlockPos { x: 0, y: 64, z: 0 },
            side: Some(Direction::North),
            slot: 0,
        };
        assert_eq!(
            transfer(world, endpoint, endpoint, 0).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            transfer(world, endpoint, endpoint, 65).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        let invalid_slot = InventoryEndpoint {
            slot: MAX_INVENTORY_SLOTS as u32,
            ..endpoint
        };
        assert_eq!(
            transfer(world, invalid_slot, endpoint, 1)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            transfer(world, endpoint, endpoint, 1).unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn public_move_checks_both_slots_count_positions_and_distinct_endpoints() {
        let world = WorldHandle::new(1, 2, 3, 4).unwrap();
        let source = BlockPos { x: 0, y: 64, z: 0 };
        let destination = BlockPos { x: 1, y: 64, z: 0 };
        assert_eq!(
            move_stack(world, source, 0, source, 0, 1)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            move_stack(world, source, 0, destination, 0, 0)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            move_stack(
                world,
                source,
                0,
                destination,
                0,
                MAX_ITEM_TRANSFER_COUNT + 1
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            move_stack(world, source, MAX_INVENTORY_SLOTS as u32, destination, 0, 1)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            move_stack(
                world,
                BlockPos {
                    x: MAX_INVENTORY_COORDINATE + 1,
                    ..source
                },
                0,
                destination,
                0,
                1,
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            move_stack(world, source, 0, destination, 0, 1)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn integer_component_reads_and_writes_validate_local_inputs_before_host_access() {
        let world = WorldHandle::new(1, 2, 3, 4).unwrap();
        let position = BlockPos { x: 0, y: 64, z: 0 };

        assert_eq!(
            read_integer_component(world, position, None, MAX_INVENTORY_SLOTS as u32, "charge")
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            read_integer_component(world, position, None, 0, "Charge")
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            read_integer_component(world, position, None, 0, "charge")
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            write_integer_component(world, position, 0, "../charge", 3)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            write_integer_component(world, position, MAX_INVENTORY_SLOTS as u32, "charge", 3)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            write_integer_component(world, position, 0, "charge", -10)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn boolean_component_reads_and_writes_validate_local_inputs_before_host_access() {
        let world = WorldHandle::new(1, 2, 3, 4).unwrap();
        let position = BlockPos { x: 0, y: 64, z: 0 };

        assert_eq!(
            read_boolean_component(world, position, None, MAX_INVENTORY_SLOTS as u32, "enabled")
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            read_boolean_component(world, position, None, 0, "Enabled")
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            read_boolean_component(world, position, None, 0, "enabled")
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            write_boolean_component(world, position, 0, "../enabled", true)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            write_boolean_component(
                world,
                position,
                MAX_INVENTORY_SLOTS as u32,
                "enabled",
                false
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            write_boolean_component(world, position, 0, "enabled", true)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn float_component_reads_and_writes_validate_local_inputs_before_host_access() {
        let world = WorldHandle::new(1, 2, 3, 4).unwrap();
        let position = BlockPos { x: 0, y: 64, z: 0 };
        assert_eq!(
            read_float_component(world, position, None, MAX_INVENTORY_SLOTS as u32, "heat")
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            read_float_component(world, position, None, 0, "../heat")
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            read_float_component(world, position, None, 0, "heat")
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            write_float_component(world, position, 0, "heat", f32::NAN)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            write_float_component(world, position, 0, "heat", f32::INFINITY)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            write_float_component(world, position, 0, "../heat", 1.0)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            write_float_component(world, position, 0, "heat", 1.0)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn custom_item_names_are_bounded_plain_values_before_host_access() {
        assert_eq!(validate_custom_name("Café 🪨"), Ok(()));
        assert_eq!(
            validate_custom_name("\nname").unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            validate_custom_name("").unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            validate_custom_name(&"🪨".repeat(MAX_CUSTOM_ITEM_NAME_CHARS + 1))
                .unwrap_err()
                .code(),
            ErrorCode::PayloadTooLarge
        );
        assert_eq!(
            validate_custom_name(&"é".repeat(MAX_CUSTOM_ITEM_NAME_BYTES / 2 + 1))
                .unwrap_err()
                .code(),
            ErrorCode::PayloadTooLarge
        );
        let world = WorldHandle::new(1, 2, 3, 4).unwrap();
        let position = BlockPos { x: 0, y: 64, z: 0 };
        assert_eq!(
            set_custom_name(
                world,
                position,
                0,
                &"x".repeat(MAX_CUSTOM_ITEM_NAME_CHARS + 1)
            )
            .unwrap_err()
            .code(),
            ErrorCode::PayloadTooLarge
        );
        assert_eq!(
            clear_custom_name(world, position, MAX_INVENTORY_SLOTS as u32)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            set_custom_name(world, position, 0, "Valid")
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn block_inventory_enchantment_ids_and_levels_are_bounded_before_host_access() {
        assert_eq!(validate_enchantment_id("minecraft:sharpness"), Ok(()));
        assert_eq!(
            validate_enchantment_id("Minecraft:sharpness")
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            validate_enchantment_id(&format!(
                "test:{}",
                "x".repeat(MAX_BLOCK_INVENTORY_ENCHANTMENT_ID_BYTES)
            ))
            .unwrap_err()
            .code(),
            ErrorCode::PayloadTooLarge
        );

        let world = WorldHandle::new(1, 2, 3, 4).unwrap();
        let position = BlockPos { x: 0, y: 64, z: 0 };
        assert_eq!(
            read_enchantment_level(
                world,
                position,
                MAX_INVENTORY_SLOTS as u32,
                "minecraft:sharpness"
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            set_enchantment_level(world, position, 0, "minecraft:sharpness", 0)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            set_enchantment_level(
                world,
                position,
                0,
                "minecraft:sharpness",
                MAX_BLOCK_INVENTORY_ENCHANTMENT_LEVEL + 1
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            remove_enchantment(world, position, 0, "../sharpness")
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            read_enchantment_level(world, position, 0, "minecraft:sharpness")
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            remove_enchantment(world, position, 0, "minecraft:sharpness")
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn block_inventory_enchantment_levels_use_the_stable_optional_u32_format() {
        assert_eq!(crate::to_bytes(&None::<u32>).unwrap(), [0]);
        let encoded = crate::to_bytes(&Some(5_u32)).unwrap();
        assert_eq!(encoded, [1, 5, 0, 0, 0]);
        assert_eq!(crate::from_bytes::<Option<u32>>(&encoded).unwrap(), Some(5));
    }

    #[test]
    fn integer_component_optional_values_use_tag_then_little_endian_i32() {
        assert_eq!(crate::to_bytes(&None::<i32>).unwrap(), [0]);
        assert_eq!(
            crate::to_bytes(&Some(-12_i32)).unwrap(),
            [1, 0xf4, 0xff, 0xff, 0xff]
        );
        assert_eq!(
            crate::from_bytes::<Option<i32>>(&[1, 0xf4, 0xff, 0xff, 0xff]).unwrap(),
            Some(-12)
        );
    }

    #[test]
    fn float_component_optional_values_use_tag_then_little_endian_f32() {
        assert_eq!(crate::to_bytes(&None::<f32>).unwrap(), [0]);
        let encoded = crate::to_bytes(&Some(1.5_f32)).unwrap();
        assert_eq!(encoded, [1, 0, 0, 0xc0, 0x3f]);
        assert_eq!(
            crate::from_bytes::<Option<f32>>(&encoded).unwrap(),
            Some(1.5)
        );
    }

    #[test]
    fn block_inventory_lore_payload_matches_schema_v1_wire_format() {
        let lines = vec!["Café 🪨".to_owned(), "§cplain".to_owned()];
        let payload = super::BlockInventoryLorePayload {
            lines: lines.clone(),
        };
        let encoded = crate::to_bytes(&payload).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_u32.to_le_bytes());
        expected.extend_from_slice(&(lines.len() as u32).to_le_bytes());
        for line in &lines {
            expected.extend_from_slice(&(line.len() as u32).to_le_bytes());
            expected.extend_from_slice(line.as_bytes());
        }
        assert_eq!(encoded, expected);
        assert_eq!(
            crate::from_bytes::<super::BlockInventoryLorePayload>(&encoded).unwrap(),
            payload
        );
        assert_eq!(MAX_BLOCK_INVENTORY_LORE_BYTES, 2088);
    }

    #[test]
    fn block_inventory_lore_validates_the_player_contract_limits() {
        assert_eq!(validate_lore_lines(&["Café 🪨".to_owned()], false), Ok(()));
        assert_eq!(validate_lore_lines(&[], true), Ok(()));
        assert_eq!(
            validate_lore_lines(&[], false).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            validate_lore_lines(
                &vec!["line".to_owned(); MAX_BLOCK_INVENTORY_LORE_LINES + 1],
                false,
            )
            .unwrap_err()
            .code(),
            ErrorCode::PayloadTooLarge
        );
        assert_eq!(
            validate_lore_line(&"é".repeat(MAX_BLOCK_INVENTORY_LORE_LINE_BYTES / 2 + 1))
                .unwrap_err()
                .code(),
            ErrorCode::PayloadTooLarge
        );
        assert_eq!(
            validate_lore_line("line\nfeed").unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn block_inventory_lore_api_validates_before_unsupported_native_import() {
        let world = WorldHandle::new(1, 2, 3, 4).unwrap();
        let position = BlockPos { x: 0, y: 64, z: 0 };
        assert_eq!(
            set_lore(world, position, 0, &[]).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            set_lore(
                world,
                position,
                MAX_INVENTORY_SLOTS as u32,
                &["line".to_owned()]
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            read_lore(world, position, MAX_INVENTORY_SLOTS as u32)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            clear_lore(world, position, MAX_INVENTORY_SLOTS as u32)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            set_lore(world, position, 0, &["line".to_owned()])
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
    }
}
