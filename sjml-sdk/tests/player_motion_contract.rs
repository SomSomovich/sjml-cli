use sjml::{from_bytes, player, to_bytes, ErrorCode, PlayerHandle};

#[test]
fn native_player_motion_and_velocity_mutation_report_unsupported_imports() {
    let handle = PlayerHandle::new(1, 2, 3, 4).unwrap();
    assert_eq!(
        player::motion(handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        player::set_velocity(handle, player::PlayerVelocity::new(0.0, 0.1, 0.0).unwrap())
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
}

#[test]
fn player_rotation_setter_validates_angles_and_uses_its_native_import() {
    let handle = PlayerHandle::new(1, 2, 3, 4).unwrap();
    for (yaw, pitch) in [
        (f32::NAN, 0.0),
        (f32::INFINITY, 0.0),
        (player::MAX_PLAYER_ROTATION_YAW + 0.1, 0.0),
        (0.0, f32::NEG_INFINITY),
        (0.0, -90.1),
        (0.0, 90.1),
    ] {
        assert_eq!(
            player::set_rotation(handle, yaw, pitch).unwrap_err().code(),
            ErrorCode::InvalidArgument,
            "yaw={yaw}, pitch={pitch} should be rejected"
        );
    }
    assert_eq!(
        player::set_rotation(handle, 270.0, -90.0)
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode,
        "valid angles should reach the native unsupported-import path"
    );
}

#[test]
fn player_motion_uses_schema_v1_velocity_and_rotation_field_order() {
    let motion = player::PlayerMotion {
        velocity_x: 1.25,
        velocity_y: -2.5,
        velocity_z: 3.75,
        yaw: 135.0,
        pitch: -22.5,
    };
    let mut expected = 1_u32.to_le_bytes().to_vec();
    for component in [motion.velocity_x, motion.velocity_y, motion.velocity_z] {
        expected.extend_from_slice(&component.to_le_bytes());
    }
    expected.extend_from_slice(&motion.yaw.to_le_bytes());
    expected.extend_from_slice(&motion.pitch.to_le_bytes());
    assert_eq!(expected.len(), 36);
    assert_eq!(to_bytes(&motion).unwrap(), expected);
    assert_eq!(
        from_bytes::<player::PlayerMotion>(&expected).unwrap(),
        motion
    );
}

#[test]
fn player_velocity_uses_schema_v1_little_endian_components() {
    let velocity = player::PlayerVelocity {
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
        from_bytes::<player::PlayerVelocity>(&expected).unwrap(),
        velocity
    );
}

#[test]
fn player_velocity_rejects_non_finite_or_out_of_range_components_before_import() {
    for (x, y, z) in [
        (f64::NAN, 0.0, 0.0),
        (0.0, f64::INFINITY, 0.0),
        (0.0, 0.0, f64::NEG_INFINITY),
        (player::MAX_PLAYER_VELOCITY_COMPONENT + 0.001, 0.0, 0.0),
        (0.0, -player::MAX_PLAYER_VELOCITY_COMPONENT - 1.0, 0.0),
    ] {
        assert_eq!(
            player::PlayerVelocity::new(x, y, z).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }

    let valid = player::PlayerVelocity::new(64.0, -64.0, 0.0).unwrap();
    assert_eq!(valid.x, 64.0);
    assert_eq!(valid.y, -64.0);
    let invalid_literal = player::PlayerVelocity {
        x: f64::NAN,
        y: 0.0,
        z: 0.0,
    };
    assert_eq!(
        player::set_velocity(PlayerHandle::new(1, 2, 3, 4).unwrap(), invalid_literal)
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
}
