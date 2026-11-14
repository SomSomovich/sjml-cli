use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering};

use sjml::{
    command, event,
    events::{self, EventPriority},
    ErrorCode, SjmlError, SjmlResult, StableDecode,
};

#[path = "../../src/world_chunk_lifecycle.rs"]
#[allow(dead_code)]
mod contract;
use contract::WorldChunkLifecycle;

static EXPECTED_DIMENSION_HASH: AtomicU32 = AtomicU32::new(0);
static EXPECTED_CHUNK_X: AtomicI32 = AtomicI32::new(0);
static EXPECTED_CHUNK_Z: AtomicI32 = AtomicI32::new(0);
static SAW_CHUNK_LOAD: AtomicBool = AtomicBool::new(false);
static SAW_CHUNK_UNLOAD: AtomicBool = AtomicBool::new(false);

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("world.chunk_loaded", EventPriority::Normal)?;
    events::subscribe("world.chunk_unloaded", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "world.chunk_loaded")]
fn observe_chunk_loaded(event: WorldChunkLifecycle) -> SjmlResult<()> {
    if matches_expected(&event) {
        SAW_CHUNK_LOAD.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[event(name = "world.chunk_unloaded")]
fn observe_chunk_unloaded(event: WorldChunkLifecycle) -> SjmlResult<()> {
    if matches_expected(&event) {
        SAW_CHUNK_UNLOAD.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct WorldChunkLifecycleTestArguments {
    operation: String,
    dimension_id: String,
    chunk_x: i32,
    chunk_z: i32,
}

#[command(name = "world.chunk_lifecycle.test")]
fn test_world_chunk_lifecycle(arguments: WorldChunkLifecycleTestArguments) -> SjmlResult<String> {
    if arguments.operation == "arm" {
        EXPECTED_DIMENSION_HASH.store(string_hash(&arguments.dimension_id), Ordering::Relaxed);
        EXPECTED_CHUNK_X.store(arguments.chunk_x, Ordering::Relaxed);
        EXPECTED_CHUNK_Z.store(arguments.chunk_z, Ordering::Relaxed);
        SAW_CHUNK_LOAD.store(false, Ordering::Relaxed);
        SAW_CHUNK_UNLOAD.store(false, Ordering::Relaxed);
        return Ok("world chunk lifecycle observer armed".to_owned());
    }
    if arguments.operation == "check" {
        let observed = SAW_CHUNK_LOAD.swap(false, Ordering::Relaxed)
            && SAW_CHUNK_UNLOAD.swap(false, Ordering::Relaxed);
        if !observed {
            return Err(failure(
                "guest did not receive both expected chunk lifecycle events",
            ));
        }
        return Ok("guest received both expected chunk lifecycle events".to_owned());
    }
    Err(failure("operation must be either arm or check"))
}

fn matches_expected(event: &WorldChunkLifecycle) -> bool {
    EXPECTED_DIMENSION_HASH.load(Ordering::Relaxed) != 0
        && string_hash(&event.dimension_id) == EXPECTED_DIMENSION_HASH.load(Ordering::Relaxed)
        && event.chunk_x == EXPECTED_CHUNK_X.load(Ordering::Relaxed)
        && event.chunk_z == EXPECTED_CHUNK_Z.load(Ordering::Relaxed)
}

fn string_hash(value: &str) -> u32 {
    value
        .bytes()
        .fold(2_166_136_261, |hash, byte| {
            (hash ^ u32::from(byte)).wrapping_mul(16_777_619)
        })
        .max(1)
}

fn failure(message: &'static str) -> SjmlError {
    SjmlError::new(ErrorCode::InvalidArgument, message)
}
