pub use sjml::{
    abi, to_bytes, Decoder, ErrorCode, PlayerHandle, SjmlError, SjmlResult, StableDecode,
    MAX_PAYLOAD_BYTES,
};

pub mod inventory {
    pub use sjml::inventory::*;
}

#[path = "../src/player_inventory.rs"]
mod player_inventory;
pub use player_inventory::read as player_inventory_read;
pub use player_inventory::{set_slot as player_inventory_set_slot, PlayerInventorySlotUpdate};

use sjml::inventory::{InventorySnapshot, ItemStackDescriptor};

#[test]
fn player_inventory_snapshot_round_trips_the_shared_inventory_schema() {
    let snapshot = InventorySnapshot {
        slots: vec![
            None,
            Some(ItemStackDescriptor {
                item_id: "minecraft:iron_ingot".to_owned(),
                count: 12,
                max_stack_size: 64,
                slot_limit: 64,
            }),
        ],
    };
    let bytes = sjml::to_bytes(&Some(snapshot.clone())).unwrap();
    assert_eq!(
        sjml::from_bytes::<Option<InventorySnapshot>>(&bytes).unwrap(),
        Some(snapshot)
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn public_player_inventory_read_fails_closed_without_a_wasm_host() {
    let player = PlayerHandle::new(1, 2, 3, 4).unwrap();
    assert_eq!(
        player_inventory::read(player).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn public_player_inventory_slot_writes_validate_then_fail_closed_without_a_wasm_host() {
    let player = PlayerHandle::new(1, 2, 3, 4).unwrap();
    assert_eq!(
        player_inventory_set_slot(player, 40, PlayerInventorySlotUpdate::Clear)
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        player_inventory_set_slot(player, 41, PlayerInventorySlotUpdate::Clear,)
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        player_inventory_set_slot(
            player,
            0,
            PlayerInventorySlotUpdate::Set {
                item_id: "minecraft:air",
                count: 1,
            },
        )
        .unwrap_err()
        .code(),
        ErrorCode::InvalidArgument
    );
}
