use sjml::{player, ErrorCode, PlayerHandle, PlayerRespawnPoint};

#[test]
fn respawn_point_constructor_enforces_dimension_coordinate_and_angle_bounds() {
    let point = PlayerRespawnPoint::new(1, 80, -2, "minecraft:overworld", 90.0, true).unwrap();
    assert_eq!(point.x, 1);
    assert_eq!(point.y, 80);
    assert_eq!(point.z, -2);
    assert_eq!(point.dimension_id, "minecraft:overworld");
    assert_eq!(point.angle, 90.0);
    assert!(point.forced);

    for invalid_dimension in [
        "overworld",
        "minecraft:",
        ":overworld",
        "Minecraft:overworld",
        "minecraft:Overworld",
    ] {
        assert_eq!(
            PlayerRespawnPoint::new(0, 64, 0, invalid_dimension, 0.0, false)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument,
            "invalid dimension should be rejected: {invalid_dimension}"
        );
    }

    let oversized_dimension = format!("mod:{}", "x".repeat(254));
    assert_eq!(
        PlayerRespawnPoint::new(0, 64, 0, oversized_dimension, 0.0, false)
            .unwrap_err()
            .code(),
        ErrorCode::PayloadTooLarge
    );

    for invalid in [
        PlayerRespawnPoint {
            x: 30_000_001,
            y: 64,
            z: 0,
            dimension_id: "minecraft:overworld".to_owned(),
            angle: 0.0,
            forced: false,
        },
        PlayerRespawnPoint {
            x: 0,
            y: i32::MIN,
            z: 0,
            dimension_id: "minecraft:overworld".to_owned(),
            angle: 0.0,
            forced: false,
        },
        PlayerRespawnPoint {
            x: 0,
            y: 64,
            z: 0,
            dimension_id: "minecraft:overworld".to_owned(),
            angle: f32::NAN,
            forced: false,
        },
        PlayerRespawnPoint {
            x: 0,
            y: 64,
            z: 0,
            dimension_id: "minecraft:overworld".to_owned(),
            angle: 36_001.0,
            forced: false,
        },
    ] {
        assert_eq!(
            player::set_respawn_point(PlayerHandle::new(1, 2, 3, 4).unwrap(), invalid)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }
}

#[test]
fn respawn_point_has_a_stable_versioned_round_trip() {
    let point = PlayerRespawnPoint::new(3, 85, -4, "example:moon/base", -37.5, false).unwrap();
    let encoded = sjml::to_bytes(&point).unwrap();
    assert_eq!(
        sjml::from_bytes::<PlayerRespawnPoint>(&encoded).unwrap(),
        point
    );

    for snapshot in [None, Some(point)] {
        let encoded = sjml::to_bytes(&snapshot).unwrap();
        assert_eq!(
            sjml::from_bytes::<Option<PlayerRespawnPoint>>(&encoded).unwrap(),
            snapshot
        );
    }
}

#[test]
fn respawn_point_host_operations_fail_closed_without_a_wasm_host() {
    let handle = PlayerHandle::new(1, 2, 3, 4).unwrap();
    let point = PlayerRespawnPoint::new(3, 85, -4, "minecraft:overworld", 0.0, false).unwrap();
    assert_eq!(
        player::respawn_point(handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        player::set_respawn_point(handle, point).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        player::clear_respawn_point(handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
}
