use sjml::{entity, from_bytes, to_bytes, EntityHandle, ErrorCode};

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn native_entity_api_reports_unsupported_imports() {
    let handle = EntityHandle::new(1, 2, 3, 4).unwrap();

    assert_eq!(
        entity::loaded().unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        entity::snapshot(handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
}

#[test]
fn entity_snapshot_uses_schema_v1_stable_field_encoding() {
    let snapshot = entity::EntitySnapshot {
        uuid: "00112233-4455-6677-8899-aabbccddeeff".to_owned(),
        entity_type_id: "minecraft:zombie".to_owned(),
        dimension_id: "minecraft:overworld".to_owned(),
        x: 1.25,
        y: -2.5,
        z: 3.75,
    };

    let encoded = to_bytes(&snapshot).expect("encode entity snapshot");
    let mut expected = 1_u32.to_le_bytes().to_vec();
    for value in [
        snapshot.uuid.as_str(),
        snapshot.entity_type_id.as_str(),
        snapshot.dimension_id.as_str(),
    ] {
        expected.extend_from_slice(&(value.len() as u32).to_le_bytes());
        expected.extend_from_slice(value.as_bytes());
    }
    for coordinate in [snapshot.x, snapshot.y, snapshot.z] {
        expected.extend_from_slice(&coordinate.to_le_bytes());
    }

    assert_eq!(encoded, expected);
    assert_eq!(
        from_bytes::<entity::EntitySnapshot>(&encoded).unwrap(),
        snapshot
    );
}
