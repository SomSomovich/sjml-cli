use sjml::{player, world, BlockPos, ErrorCode, PlayerHandle, WorldHandle};

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
        world::block_states(handle, &[BlockPos { x: 1, y: 64, z: 2 }])
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
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
