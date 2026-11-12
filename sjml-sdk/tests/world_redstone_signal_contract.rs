use sjml::{world, BlockPos, ErrorCode, WorldHandle, MAX_WORLD_REDSTONE_SIGNAL};

#[test]
fn redstone_signal_uses_the_bounded_vanilla_range_and_stable_optional_encoding() {
    assert_eq!(MAX_WORLD_REDSTONE_SIGNAL, 15);
    assert_eq!(sjml::to_bytes(&Some(0u8)).unwrap(), [1, 0]);
    assert_eq!(sjml::to_bytes(&Some(15u8)).unwrap(), [1, 15]);
    assert_eq!(sjml::to_bytes(&Option::<u8>::None).unwrap(), [0]);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn redstone_signal_query_rejects_out_of_bounds_xz_before_requiring_the_wasm_host() {
    let handle = WorldHandle::new(1, 2, 3, 4).unwrap();
    assert_eq!(
        world::best_neighbor_signal(
            handle,
            BlockPos {
                x: 30_000_001,
                y: 64,
                z: 0,
            },
        )
        .unwrap_err()
        .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        world::best_neighbor_signal(handle, BlockPos { x: 0, y: 64, z: 0 })
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
}
