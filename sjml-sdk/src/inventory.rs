//! Bounded item inventory access on loaded blocks. Reads require `world.read` and
//! `inventory.read`; component writes require `world.read` and `inventory.write`.

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

/// Stable value snapshot of every handler slot. Empty slots are `None`.
#[derive(Debug, Clone, Eq, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct InventorySnapshot {
    pub slots: Vec<Option<ItemStackDescriptor>>,
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

fn validate_slot(slot: u32) -> SjmlResult<()> {
    if slot as usize >= MAX_INVENTORY_SLOTS {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "inventory slot must be within the 4096-slot ABI limit",
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
    let capacity = crate::MAX_PAYLOAD_BYTES as u32;
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
            "host rejected the inventory read",
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
        "inventory reads require the wasm32-unknown-unknown target",
    )
}
