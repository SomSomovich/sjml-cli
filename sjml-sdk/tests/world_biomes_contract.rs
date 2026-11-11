use sjml::{world, BlockPos, ErrorCode, WorldHandle};

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn biome_lookup_requires_the_wasm_host() {
    let handle = WorldHandle::new(1, 2, 3, 4).unwrap();
    assert_eq!(
        world::biome_at(handle, BlockPos { x: 1, y: 64, z: 2 })
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
}
