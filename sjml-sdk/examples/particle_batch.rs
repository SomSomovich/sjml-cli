use sjml::inventory::BlockPos;
use sjml::{world, ErrorCode, SjmlError, SjmlResult};

#[allow(dead_code)]
fn emit_particle_batch() -> SjmlResult<String> {
    let overworld = world::get("minecraft:overworld")?.ok_or_else(|| {
        SjmlError::new(ErrorCode::InvalidArgument, "the Overworld is unavailable")
    })?;
    let options = world::ParticleSpawnOptions::new(32, 0.25, 0.5, 0.25, 0.2)?;
    world::spawn_particles(
        overworld,
        BlockPos { x: 0, y: 80, z: 0 },
        "particle_batch:ruby_spark",
        options,
    )?;
    Ok("emitted a bounded particle batch".to_owned())
}

fn main() {}
