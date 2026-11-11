use sjml::block_entity::{BlockEntitySnapshot, NbtValue};
use sjml::{BlockPos, ErrorCode, WorldHandle};

#[test]
fn snapshot_round_trips_as_versioned_detached_values() {
    let value = BlockEntitySnapshot {
        type_id: "minecraft:chest".into(),
        dimension_id: "minecraft:overworld".into(),
        position: BlockPos { x: -2, y: 64, z: 9 },
        block_state: "minecraft:chest[facing=north]".into(),
    };
    assert_eq!(
        sjml::from_bytes::<BlockEntitySnapshot>(&sjml::to_bytes(&value).unwrap()).unwrap(),
        value
    );
}

#[test]
fn nbt_snapshot_round_trips_typed_values() {
    let value = NbtValue::Compound(vec![
        ("count".into(), NbtValue::Int(42)),
        ("name".into(), NbtValue::String("test".into())),
    ]);
    assert_eq!(
        sjml::from_bytes::<NbtValue>(&sjml::to_bytes(&value).unwrap()).unwrap(),
        value
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn native_world_block_entity_reports_unsupported_import() {
    let world = WorldHandle::new(1, 2, 3, 4).unwrap();
    assert_eq!(
        world
            .block_entity(BlockPos { x: 0, y: 0, z: 0 })
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
}

#[test]
fn nbt_rejects_unknown_tag_and_excessive_depth() {
    let mut invalid = 1u32.to_le_bytes().to_vec();
    invalid.push(99);
    assert_eq!(
        sjml::from_bytes::<NbtValue>(&invalid).unwrap_err().code(),
        ErrorCode::InvalidArgument
    );
    let mut value = NbtValue::Int(0);
    for _ in 0..64 {
        value = NbtValue::List(vec![value]);
    }
    assert_eq!(
        sjml::to_bytes(&value).unwrap_err().code(),
        ErrorCode::PayloadTooLarge
    );
}

#[test]
fn nbt_rejects_duplicate_compound_keys_and_mixed_lists() {
    let duplicate = NbtValue::Compound(vec![
        ("x".into(), NbtValue::Int(1)),
        ("x".into(), NbtValue::Int(2)),
    ]);
    assert_eq!(
        sjml::to_bytes(&duplicate).unwrap_err().code(),
        ErrorCode::InvalidArgument
    );
    let mixed = NbtValue::List(vec![NbtValue::Byte(1), NbtValue::Int(2)]);
    assert_eq!(
        sjml::to_bytes(&mixed).unwrap_err().code(),
        ErrorCode::InvalidArgument
    );
}
