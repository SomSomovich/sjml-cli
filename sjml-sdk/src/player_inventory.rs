//! Read-only snapshots of a live player's inventory.
//!
//! `read` requires both `entity.read` and `inventory.read` and is available only to
//! logical-server modules. It returns detached item IDs, counts, and slot bounds; it
//! never exposes Minecraft objects or allows inventory mutation.

#[cfg(target_arch = "wasm32")]
use crate::abi::{allocate_host_input, deallocate_host_input};
use crate::inventory::InventorySnapshot;
#[cfg(any(target_arch = "wasm32", test))]
use crate::inventory::{ItemStackDescriptor, MAX_INVENTORY_SLOTS};
#[cfg(target_arch = "wasm32")]
use crate::to_bytes;
#[cfg(target_arch = "wasm32")]
use crate::{Decoder, StableDecode};
use crate::{ErrorCode, PlayerHandle, SjmlError, SjmlResult};

/// Maximum number of slots returned by one player inventory query.
pub const MAX_PLAYER_INVENTORY_SLOTS: usize = 128;

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_inventory_v1")]
extern "C" {
    #[link_name = "read_player"]
    fn imported_read_player(player: i32, player_length: i32, output: i32, capacity: i32) -> i32;
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
    let Some((namespace, path)) = stack.item_id.split_once(':') else {
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
        "player inventory reads require the wasm32-unknown-unknown target",
    )
}

#[cfg(test)]
mod tests {
    #[cfg(not(target_arch = "wasm32"))]
    use super::read;
    use super::{validate_inventory, MAX_PLAYER_INVENTORY_SLOTS};
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

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_read_fails_closed_after_typed_handle_validation() {
        let player = PlayerHandle::new(1, 2, 3, 4).unwrap();
        assert_eq!(
            read(player).unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
    }
}
