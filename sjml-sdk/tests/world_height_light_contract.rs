use sjml::{
    world::{self, HeightmapType, LightSnapshot},
    BlockPos, ErrorCode, WorldHandle,
};

#[test]
fn heightmap_tags_and_light_snapshot_have_stable_world_v1_encoding() {
    assert_eq!(HeightmapType::WorldSurface as i32, 0);
    assert_eq!(HeightmapType::OceanFloor as i32, 1);
    assert_eq!(HeightmapType::MotionBlocking as i32, 2);
    assert_eq!(HeightmapType::MotionBlockingNoLeaves as i32, 3);

    let light = Some(LightSnapshot { sky: 13, block: 7 });
    let bytes = sjml::to_bytes(&light).unwrap();
    assert_eq!(bytes, [1, 1, 0, 0, 0, 13, 7]);
    assert_eq!(
        sjml::from_bytes::<Option<LightSnapshot>>(&bytes).unwrap(),
        light
    );

    let absent_height = sjml::to_bytes(&Option::<i32>::None).unwrap();
    assert_eq!(absent_height, [0]);
    assert_eq!(
        sjml::from_bytes::<Option<i32>>(&absent_height).unwrap(),
        None
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn heightmap_and_light_queries_require_the_wasm_host_and_reject_out_of_bounds_xz() {
    let handle = WorldHandle::new(1, 2, 3, 4).unwrap();
    assert_eq!(
        world::heightmap(handle, HeightmapType::WorldSurface, 0, 0)
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        world::heightmap(handle, HeightmapType::WorldSurface, 30_000_001, 0)
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        world::light_at(handle, BlockPos { x: 0, y: 64, z: 0 })
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        world::light_at(
            handle,
            BlockPos {
                x: 0,
                y: 64,
                z: -30_000_001,
            },
        )
        .unwrap_err()
        .code(),
        ErrorCode::InvalidArgument
    );
}
