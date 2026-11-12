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
pub use player_inventory::{
    clear_item_custom_name as player_inventory_clear_item_custom_name,
    set_damage as player_inventory_set_damage,
    set_item_custom_name as player_inventory_set_item_custom_name,
    set_slot as player_inventory_set_slot, PlayerInventorySlotUpdate,
};

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

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn public_player_inventory_durability_write_validates_then_fails_closed_without_a_wasm_host() {
    let player = PlayerHandle::new(1, 2, 3, 4).unwrap();
    assert_eq!(
        player_inventory_set_damage(player, 0, 7)
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        player_inventory_set_damage(player, 41, 7)
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn public_player_inventory_item_custom_name_set_and_clear_validate_then_fail_closed() {
    let player = PlayerHandle::new(1, 2, 3, 4).unwrap();
    assert_eq!(
        player_inventory_set_item_custom_name(player, 0, "Café 🪨")
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        player_inventory_clear_item_custom_name(player, 40)
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        player_inventory_set_item_custom_name(player, 41, "name")
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        player_inventory_clear_item_custom_name(player, 41)
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        player_inventory_set_item_custom_name(player, 0, "")
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        player_inventory_set_item_custom_name(player, 0, "invalid\nname")
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
}
