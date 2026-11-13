use sjml::{entity, from_bytes, to_bytes, EntityHandle, ErrorCode};

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn native_health_api_reports_unsupported_imports_and_validates_heal_bounds() {
    let handle = EntityHandle::new(1, 2, 3, 4).unwrap();

    assert_eq!(
        entity::vitals(handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        entity::heal(handle, 1.0).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    for invalid in [
        0.0,
        -1.0,
        f32::NAN,
        f32::INFINITY,
        entity::MAX_ENTITY_HEAL_AMOUNT + 0.01,
    ] {
        assert_eq!(
            entity::heal(handle, invalid).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }
}

#[test]
fn vitals_uses_the_schema_v1_little_endian_field_order() {
    let vitals = entity::EntityVitals {
        health: 12.5,
        max_health: 20.0,
        absorption: 3.25,
    };
    let mut expected = 1_u32.to_le_bytes().to_vec();
    expected.extend_from_slice(&12.5_f32.to_le_bytes());
    expected.extend_from_slice(&20.0_f32.to_le_bytes());
    expected.extend_from_slice(&3.25_f32.to_le_bytes());

    let encoded = to_bytes(&vitals).expect("encode entity vitals");
    assert_eq!(encoded, expected);
    assert_eq!(
        from_bytes::<entity::EntityVitals>(&encoded).unwrap(),
        vitals
    );
    assert_eq!(entity::MAX_ENTITY_VITALS_VALUE, 1_000_000.0);
    assert_eq!(entity::HEALTH_WRITE_CAPABILITY, "entity.health.write");
}
