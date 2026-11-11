pub use sjml::{
    Decoder, ErrorCode, MAX_PAYLOAD_BYTES, PlayerHandle, SjmlError, SjmlResult, StableDecode, abi,
    to_bytes,
};

pub mod inventory {
    pub use sjml::inventory::*;
}

#[path = "../src/player_inventory.rs"]
mod player_inventory;
pub use player_inventory::read as player_inventory_read;

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
