use sjml::inventory::{
    BlockPos, Direction, InventorySnapshot, ItemStackDescriptor, ItemStackSnapshot,
};
use sjml::{inventory, ErrorCode, WorldHandle};

#[test]
fn inventory_records_round_trip_with_versioned_detached_values() {
    let snapshot = InventorySnapshot {
        slots: vec![
            None,
            Some(ItemStackDescriptor {
                item_id: "minecraft:stone".to_owned(),
                count: 70,
                max_stack_size: 64,
                slot_limit: 32,
            }),
        ],
    };
    let encoded = sjml::to_bytes(&Some(snapshot.clone())).unwrap();
    assert_eq!(
        sjml::from_bytes::<Option<InventorySnapshot>>(&encoded).unwrap(),
        Some(snapshot)
    );
}

#[test]
fn one_slot_stack_snapshot_round_trips_its_versioned_bounded_fields() {
    let snapshot = ItemStackSnapshot {
        item_id: "minecraft:diamond_sword".to_owned(),
        count: 1,
        max_stack_size: 1,
        slot_limit: 64,
        damage: Some(37),
        max_damage: Some(1561),
        custom_name: Some("Café 🗡".to_owned()),
    };
    let encoded = sjml::to_bytes(&Some(snapshot.clone())).unwrap();
    let mut expected = vec![1]; // Some
    expected.extend_from_slice(&1u32.to_le_bytes()); // ItemStackSnapshot schema
    expected.extend_from_slice(&(snapshot.item_id.len() as u32).to_le_bytes());
    expected.extend_from_slice(snapshot.item_id.as_bytes());
    for value in [snapshot.count, snapshot.max_stack_size, snapshot.slot_limit] {
        expected.extend_from_slice(&value.to_le_bytes());
    }
    expected.push(1);
    expected.extend_from_slice(&37u32.to_le_bytes());
    expected.push(1);
    expected.extend_from_slice(&1561u32.to_le_bytes());
    expected.push(1);
    expected
        .extend_from_slice(&(snapshot.custom_name.as_ref().unwrap().len() as u32).to_le_bytes());
    expected.extend_from_slice(snapshot.custom_name.as_ref().unwrap().as_bytes());
    assert_eq!(encoded, expected);
    assert_eq!(
        sjml::from_bytes::<Option<ItemStackSnapshot>>(&encoded).unwrap(),
        Some(snapshot)
    );

    let without_optional_fields = ItemStackSnapshot {
        item_id: "minecraft:stone".to_owned(),
        count: 4,
        max_stack_size: 64,
        slot_limit: 64,
        damage: None,
        max_damage: None,
        custom_name: None,
    };
    assert_eq!(
        sjml::from_bytes::<Option<ItemStackSnapshot>>(
            &sjml::to_bytes(&Some(without_optional_fields.clone())).unwrap()
        )
        .unwrap(),
        Some(without_optional_fields)
    );
}

#[test]
fn position_and_sided_option_have_stable_encodings() {
    let position = BlockPos { x: -2, y: 64, z: 9 };
    assert_eq!(
        sjml::from_bytes::<BlockPos>(&sjml::to_bytes(&position).unwrap()).unwrap(),
        position
    );
    for direction in [
        Direction::Down,
        Direction::Up,
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
    ] {
        assert_eq!(
            sjml::from_bytes::<Option<Direction>>(&sjml::to_bytes(&Some(direction)).unwrap())
                .unwrap(),
            Some(direction)
        );
    }
    assert_eq!(sjml::to_bytes(&Option::<Direction>::None).unwrap(), vec![0]);
}

#[test]
fn direction_decode_rejects_unknown_discriminants() {
    let encoded = 6u32.to_le_bytes();
    assert_eq!(
        sjml::from_bytes::<Direction>(&encoded).unwrap_err().code(),
        ErrorCode::InvalidArgument
    );
}

#[test]
fn optional_string_component_values_round_trip_with_strict_stable_encoding() {
    for value in [None, Some(String::from("found in a cave"))] {
        let encoded = sjml::to_bytes(&value).unwrap();
        assert_eq!(sjml::from_bytes::<Option<String>>(&encoded).unwrap(), value);
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn native_inventory_api_reports_unsupported_import() {
    let world = WorldHandle::new(1, 2, 3, 4).unwrap();
    assert_eq!(
        inventory::read(world, BlockPos { x: 0, y: 0, z: 0 }, None)
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode,
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn native_one_slot_read_checks_slot_bounds_before_host_access() {
    let world = WorldHandle::new(1, 2, 3, 4).unwrap();
    let position = BlockPos { x: 0, y: 64, z: 0 };
    assert_eq!(
        inventory::read_stack(world, position, None, 4096)
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument,
    );
    assert_eq!(
        inventory::read_stack(world, position, None, 0)
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode,
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn native_component_api_checks_bounds_before_reporting_unsupported_import() {
    let world = WorldHandle::new(1, 2, 3, 4).unwrap();
    let position = BlockPos { x: 0, y: 0, z: 0 };
    assert_eq!(
        inventory::read_string_component(world, position, None, 0, "origin")
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode,
    );
    assert_eq!(
        inventory::read_string_component(world, position, None, 0, "Invalid")
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument,
    );
    assert_eq!(
        inventory::write_string_component(world, position, 4096, "origin", "value")
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument,
    );
    assert_eq!(
        inventory::write_string_component(world, position, 0, "origin", &"🪨".repeat(1025))
            .unwrap_err()
            .code(),
        ErrorCode::PayloadTooLarge,
    );
}
