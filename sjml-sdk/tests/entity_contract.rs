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
        entity::riding_snapshot(handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        entity::start_riding(handle, handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        entity::stop_riding(handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        entity::riding_snapshot(handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        entity::start_riding(handle, handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        entity::stop_riding(handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        entity::active_effects(handle).unwrap_err().code(),
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
        entity::rotation(handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        entity::set_rotation(handle, entity::EntityRotation::new(180.0, -35.0).unwrap())
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        entity::set_rotation(
            handle,
            entity::EntityRotation {
                yaw: f32::INFINITY,
                pitch: 0.0,
            },
        )
        .unwrap_err()
        .code(),
        ErrorCode::InvalidArgument
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
        entity::is_ai_enabled(handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        entity::set_ai_enabled(handle, false).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
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
        entity::custom_name(handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        entity::set_custom_name(handle, "plain name")
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        entity::clear_custom_name(handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        entity::is_custom_name_visible(handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        entity::set_custom_name_visible(handle, true)
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        entity::set_custom_name(handle, "").unwrap_err().code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        entity::set_custom_name(handle, "bad\nname")
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        entity::set_custom_name(handle, &"😀".repeat(257))
            .unwrap_err()
            .code(),
        ErrorCode::PayloadTooLarge
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
fn active_effect_snapshot_value_uses_the_host_schema_field_order() {
    let effect = entity::ActiveEffect {
        effect_id: "minecraft:speed".to_owned(),
        amplifier: 3,
        remaining_duration_ticks: -1,
    };
    let mut expected = 1_u32.to_le_bytes().to_vec();
    let id = effect.effect_id.as_bytes();
    expected.extend_from_slice(&(id.len() as u32).to_le_bytes());
    expected.extend_from_slice(id);
    expected.extend_from_slice(&effect.amplifier.to_le_bytes());
    expected.extend_from_slice(&effect.remaining_duration_ticks.to_le_bytes());
    assert_eq!(to_bytes(&effect).unwrap(), expected);
    assert_eq!(
        from_bytes::<entity::ActiveEffect>(&expected).unwrap(),
        effect
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
fn riding_snapshot_uses_schema_v1_stable_field_encoding_and_validates_relationships() {
    let vehicle = entity::EntityRidingRelationship {
        uuid: "00112233-4455-6677-8899-aabbccddeeff".to_owned(),
        entity_type_id: "minecraft:boat".to_owned(),
        is_player: false,
    };
    let player = entity::EntityRidingRelationship {
        uuid: "10112233-4455-6677-8899-aabbccddeeff".to_owned(),
        entity_type_id: "minecraft:player".to_owned(),
        is_player: true,
    };
    let passenger = entity::EntityRidingRelationship {
        uuid: "00112233-4455-6677-8899-aabbccdde001".to_owned(),
        entity_type_id: "minecraft:pig".to_owned(),
        is_player: false,
    };
    let snapshot = entity::EntityRidingSnapshot {
        vehicle: Some(vehicle.clone()),
        passengers: vec![passenger.clone(), player.clone()],
    };
    // The record contains one vehicle option followed by a separately sized passenger vector.
    let mut expected = 1_u32.to_le_bytes().to_vec();
    expected.push(1);
    append_relationship(&mut expected, &vehicle);
    expected.extend_from_slice(&2_u32.to_le_bytes());
    append_relationship(&mut expected, &passenger);
    append_relationship(&mut expected, &player);
    assert_eq!(to_bytes(&snapshot).unwrap(), expected);
    assert_eq!(
        from_bytes::<entity::EntityRidingSnapshot>(&expected).unwrap(),
        snapshot
    );

    let unsorted = entity::EntityRidingSnapshot {
        vehicle: None,
        passengers: vec![player.clone(), passenger.clone()],
    };
    assert_eq!(
        unsorted.validate().unwrap_err().code(),
        ErrorCode::InvalidArgument
    );
    let too_many = entity::EntityRidingSnapshot {
        vehicle: None,
        passengers: vec![passenger; entity::MAX_ENTITY_PASSENGERS + 1],
    };
    assert_eq!(
        too_many.validate().unwrap_err().code(),
        ErrorCode::PayloadTooLarge
    );
}

fn append_relationship(output: &mut Vec<u8>, relationship: &entity::EntityRidingRelationship) {
    output.extend_from_slice(&1_u32.to_le_bytes());
    for value in [
        relationship.uuid.as_str(),
        relationship.entity_type_id.as_str(),
    ] {
        output.extend_from_slice(&(value.len() as u32).to_le_bytes());
        output.extend_from_slice(value.as_bytes());
    }
    output.push(u8::from(relationship.is_player));
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

#[test]
fn entity_rotation_uses_schema_v1_yaw_then_pitch_and_checks_finite_bounds() {
    let rotation = entity::EntityRotation {
        yaw: -270.0,
        pitch: 45.5,
    };
    let mut expected = 1_u32.to_le_bytes().to_vec();
    expected.extend_from_slice(&rotation.yaw.to_le_bytes());
    expected.extend_from_slice(&rotation.pitch.to_le_bytes());
    assert_eq!(expected.len(), 12);
    assert_eq!(to_bytes(&rotation).unwrap(), expected);
    assert_eq!(
        from_bytes::<entity::EntityRotation>(&expected).unwrap(),
        rotation
    );

    assert_eq!(
        entity::EntityRotation::new(36_000.0, -90.0).unwrap().yaw,
        36_000.0
    );
    assert_eq!(
        entity::EntityRotation::new(f32::NAN, 0.0)
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        entity::EntityRotation::new(36_000.1, 0.0)
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        entity::EntityRotation::new(0.0, f32::NEG_INFINITY)
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        entity::EntityRotation::new(0.0, 90.01).unwrap_err().code(),
        ErrorCode::InvalidArgument
    );
}

#[test]
fn entity_glowing_and_silent_writes_use_separate_capabilities() {
    assert_eq!(entity::GLOWING_WRITE_CAPABILITY, "entity.glowing.write");
    assert_eq!(entity::STATE_WRITE_CAPABILITY, "entity.state.write");
}
