use sjml::{world, ErrorCode, WorldHandle, WorldSpawnPoint, MAX_WORLD_BLOCK_COORDINATE};

#[test]
fn spawn_point_constructor_bounds_horizontal_coordinates_and_angle() {
    let point = WorldSpawnPoint::new(-12, -2048, 24, -90.0).unwrap();
    assert_eq!((point.x, point.y, point.z), (-12, -2048, 24));
    assert_eq!(point.angle, -90.0);
    assert!(WorldSpawnPoint::new(
        MAX_WORLD_BLOCK_COORDINATE,
        0,
        -MAX_WORLD_BLOCK_COORDINATE,
        0.0
    )
    .is_ok());

    for invalid in [
        (MAX_WORLD_BLOCK_COORDINATE + 1, 0, 0, 0.0),
        (0, 0, -MAX_WORLD_BLOCK_COORDINATE - 1, 0.0),
        (0, 0, 0, f32::NAN),
        (0, 0, 0, f32::INFINITY),
    ] {
        assert_eq!(
            WorldSpawnPoint::new(invalid.0, invalid.1, invalid.2, invalid.3)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }
}

#[test]
fn spawn_point_host_operations_fail_closed_without_a_wasm_host() {
    let handle = WorldHandle::new(1, 2, 3, 4).unwrap();
    let point = WorldSpawnPoint::new(4, 72, -9, 30.0).unwrap();
    assert_eq!(
        world::spawn_point(handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        world::set_spawn_point(handle, point).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
}

#[test]
fn invalid_spawn_point_fields_are_rejected_before_host_access() {
    let handle = WorldHandle::new(1, 2, 3, 4).unwrap();
    for point in [
        WorldSpawnPoint {
            x: -MAX_WORLD_BLOCK_COORDINATE - 1,
            y: 64,
            z: 0,
            angle: 0.0,
        },
        WorldSpawnPoint {
            x: 0,
            y: 64,
            z: 0,
            angle: f32::NEG_INFINITY,
        },
    ] {
        assert_eq!(
            world::set_spawn_point(handle, point).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }
}
