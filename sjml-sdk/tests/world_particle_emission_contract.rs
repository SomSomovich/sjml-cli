use sjml::inventory::BlockPos;
use sjml::world::{self, ParticleSpawnOptions};
use sjml::{ErrorCode, WorldHandle};

#[test]
fn particle_options_enforce_per_call_bounds() {
    assert!(ParticleSpawnOptions::new(1, 0.0, 0.0, 0.0, 0.0).is_ok());
    assert!(ParticleSpawnOptions::new(64, 8.0, 8.0, 8.0, 8.0).is_ok());
    for invalid in [
        ParticleSpawnOptions::new(0, 0.0, 0.0, 0.0, 0.0),
        ParticleSpawnOptions::new(65, 0.0, 0.0, 0.0, 0.0),
        ParticleSpawnOptions::new(1, -0.1, 0.0, 0.0, 0.0),
        ParticleSpawnOptions::new(1, 8.1, 0.0, 0.0, 0.0),
        ParticleSpawnOptions::new(1, f32::NAN, 0.0, 0.0, 0.0),
        ParticleSpawnOptions::new(1, 0.0, 0.0, 0.0, f32::INFINITY),
    ] {
        assert_eq!(invalid.unwrap_err().code(), ErrorCode::InvalidArgument);
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn particle_emission_validates_before_host_import_and_fails_closed_natively() {
    let world = WorldHandle::new(1, 2, 3, 4).unwrap();
    let position = BlockPos { x: 1, y: 64, z: 2 };
    let options = ParticleSpawnOptions::new(8, 0.25, 0.5, 0.25, 0.2).unwrap();

    assert_eq!(
        world::spawn_particles(world, position, "Bad:particle", options)
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        world::spawn_particles(
            world,
            BlockPos {
                x: 0,
                y: i32::MAX,
                z: 0
            },
            "minecraft:flame",
            options,
        )
        .unwrap_err()
        .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        world::spawn_particles(world, position, "minecraft:flame", options)
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        world::spawn_particle(world, position, "minecraft:flame")
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
}
