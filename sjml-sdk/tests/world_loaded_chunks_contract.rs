use sjml::{world, BlockPos, ChunkPos, ErrorCode, WorldHandle};

#[test]
fn loaded_chunk_positions_keep_a_detached_stable_schema() {
    let position = ChunkPos { x: -123, z: 456 };
    let bytes = sjml::to_bytes(&position).unwrap();
    assert_eq!(1_u32.to_le_bytes(), bytes[0..4]);
    assert_eq!(position, sjml::from_bytes::<ChunkPos>(&bytes).unwrap());
}

#[test]
fn loaded_chunk_query_validates_bounds_before_calling_the_host() {
    let world = WorldHandle::new(1, 2, 3, 4).unwrap();
    let center = BlockPos { x: 0, y: 64, z: 0 };
    assert_eq!(
        ErrorCode::InvalidArgument,
        world::loaded_chunks(world, center, world::MAX_WORLD_CHUNK_RADIUS + 1)
            .unwrap_err()
            .code()
    );
    assert_eq!(
        ErrorCode::InvalidArgument,
        world::loaded_chunks(
            world,
            BlockPos {
                x: 30_000_001,
                y: 64,
                z: 0,
            },
            0,
        )
        .unwrap_err()
        .code()
    );
    assert_eq!(
        ErrorCode::UnknownErrorCode,
        world::loaded_chunks(world, center, 0).unwrap_err().code(),
        "native target fails closed instead of fabricating loaded chunks"
    );
}
