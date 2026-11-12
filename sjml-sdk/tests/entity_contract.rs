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
    assert_eq!(
        entity::discard(handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        entity::velocity(handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        entity::set_velocity(handle, entity::EntityVelocity::new(0.0, 0.1, 0.0).unwrap())
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        entity::fire_ticks(handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        entity::set_fire_ticks(handle, entity::MAX_ENTITY_FIRE_TICKS)
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        entity::set_fire_ticks(handle, entity::MAX_ENTITY_FIRE_TICKS + 1)
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        entity::is_glowing(handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        entity::set_glowing(handle, true).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        entity::is_silent(handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        entity::set_silent(handle, false).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        entity::damage(handle, 1.0).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        entity::remove_effect(handle, "minecraft:glowing")
            .unwrap_err()
            .code(),
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

#[test]
fn entity_velocity_uses_schema_v1_little_endian_component_encoding() {
    let velocity = entity::EntityVelocity {
        x: 1.25,
        y: -2.5,
        z: 3.75,
    };
    let mut expected = 1_u32.to_le_bytes().to_vec();
    for component in [velocity.x, velocity.y, velocity.z] {
        expected.extend_from_slice(&component.to_le_bytes());
    }
    assert_eq!(expected.len(), 28);
    assert_eq!(to_bytes(&velocity).unwrap(), expected);
    assert_eq!(
        from_bytes::<entity::EntityVelocity>(&expected).unwrap(),
        velocity
    );
}
