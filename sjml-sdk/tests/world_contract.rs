use sjml::{from_bytes, player, to_bytes, world, BlockPos, ErrorCode, PlayerHandle, WorldHandle};

#[test]
fn malformed_dimension_ids_are_rejected_before_host_access() {
    for invalid in [
        "overworld",
        "Minecraft:overworld",
        "minecraft:",
        ":world",
        "a:b:c",
        "a:bad path",
    ] {
        assert_eq!(
            world::get(invalid).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn native_world_api_reports_unsupported_imports() {
    let handle = WorldHandle::new(1, 2, 3, 4).unwrap();
    assert_eq!(
        world::get("minecraft:overworld").unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        world::loaded().unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        world::dimension_id(handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        world::game_time(handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        world::emit_game_event(handle, BlockPos { x: 1, y: 64, z: 2 }, "minecraft:step")
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        world::daylight_cycle_enabled(handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        world::game_rule(handle, world::WorldGameRule::KeepInventory)
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        world::set_game_rule(handle, world::WorldGameRule::KeepInventory, true)
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
    for enabled in [false, true] {
        assert_eq!(
            world::set_daylight_cycle_enabled(handle, enabled)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
    }
    assert_eq!(
        world::block_states(handle, &[BlockPos { x: 1, y: 64, z: 2 }])
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        world::block_properties(handle, BlockPos { x: 1, y: 64, z: 2 })
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn game_event_ids_are_validated_before_the_host_import() {
    let handle = WorldHandle::new(1, 2, 3, 4).unwrap();
    let position = BlockPos { x: 1, y: 64, z: 2 };
    for invalid in [
        "Minecraft:step",
        "minecraft:bad path",
        "a:b:c",
        "minecraft:",
    ] {
        assert_eq!(
            world::emit_game_event(handle, position, invalid)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }
}

#[test]
fn world_batch_limit_is_checked_before_host_access() {
    let handle = WorldHandle::new(1, 2, 3, 4).unwrap();
    let positions = vec![BlockPos { x: 0, y: 0, z: 0 }; world::MAX_WORLD_BLOCK_STATES + 1];
    assert_eq!(
        world::block_states(handle, &positions).unwrap_err().code(),
        ErrorCode::PayloadTooLarge
    );
}

#[test]
fn block_state_snapshots_have_stable_versioned_encoding() {
    let snapshot = world::BlockStateSnapshot {
        block_id: "minecraft:stone".to_owned(),
        state: "minecraft:stone".to_owned(),
    };
    assert_eq!(
        sjml::from_bytes::<world::BlockStateSnapshot>(&sjml::to_bytes(&snapshot).unwrap()).unwrap(),
        snapshot
    );
}

#[test]
fn world_border_snapshot_has_a_stable_little_endian_schema() {
    let snapshot = world::WorldBorderSnapshot {
        center_x: 12.5,
        center_z: -8.25,
        current_diameter: 1_000.0,
        target_diameter: 500.0,
        remaining_lerp_millis: 30_000,
        warning_time_seconds: 15,
        warning_distance_blocks: 7,
        damage_buffer_blocks: 5.0,
        damage_per_block: 0.2,
    };
    let mut expected = 1_u32.to_le_bytes().to_vec();
    for value in [12.5_f64, -8.25, 1_000.0, 500.0] {
        expected.extend_from_slice(&value.to_le_bytes());
    }
    expected.extend_from_slice(&30_000_u64.to_le_bytes());
    expected.extend_from_slice(&15_i32.to_le_bytes());
    expected.extend_from_slice(&7_i32.to_le_bytes());
    expected.extend_from_slice(&5.0_f64.to_le_bytes());
    expected.extend_from_slice(&0.2_f64.to_le_bytes());
    assert_eq!(expected.len(), 68);
    assert_eq!(to_bytes(&snapshot).unwrap(), expected);
    assert_eq!(
        from_bytes::<world::WorldBorderSnapshot>(&expected).unwrap(),
        snapshot
    );
}

#[test]
fn border_api_validates_ranges_before_host_access() {
    let handle = WorldHandle::new(1, 2, 3, 4).unwrap();
    assert_eq!(
        world::border(handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        world::set_border_center(handle, f64::NAN, 0.0)
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        world::set_border_size(handle, 0.0, 0).unwrap_err().code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        world::set_border_size(handle, 1_000.0, world::MAX_WORLD_BORDER_LERP_MILLIS + 1)
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        world::set_border_warnings(handle, world::MAX_WORLD_BORDER_WARNING_SECONDS + 1, 0)
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        world::set_border_damage(handle, 0.0, f64::INFINITY)
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        world::set_border_center(handle, 1.0, 2.0)
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        world::set_border_warnings(handle, 1, 2).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn native_player_api_reports_unsupported_imports() {
    let handle = PlayerHandle::new(1, 2, 3, 4).unwrap();
    assert_eq!(
        player::online().unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        player::snapshot(handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
}
