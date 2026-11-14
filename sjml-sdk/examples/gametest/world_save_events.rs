use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU32, Ordering};

use sjml::{
    command, event,
    events::{self, EventPriority},
    ErrorCode, SjmlError, SjmlResult, StableDecode,
};

#[path = "../../src/world_save_events.rs"]
#[allow(dead_code)]
mod contract;
use contract::WorldSaveObserved;

static EXPECTED_DIMENSION_HASH: AtomicU32 = AtomicU32::new(0);
static EXPECTED_GAME_TIME: AtomicI64 = AtomicI64::new(-1);
static SAW_SAVE: AtomicBool = AtomicBool::new(false);

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("world.save_observed", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "world.save_observed")]
fn observe_world_save(event: WorldSaveObserved) -> SjmlResult<()> {
    if EXPECTED_DIMENSION_HASH.load(Ordering::Relaxed) != 0
        && string_hash(&event.dimension_id) == EXPECTED_DIMENSION_HASH.load(Ordering::Relaxed)
        && event.game_time == EXPECTED_GAME_TIME.load(Ordering::Relaxed)
    {
        SAW_SAVE.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct WorldSaveTestArguments {
    operation: String,
    dimension_id: String,
    game_time: i32,
}

#[command(name = "world.save_observed.test")]
fn test_world_save_observed(arguments: WorldSaveTestArguments) -> SjmlResult<String> {
    match arguments.operation.as_str() {
        "arm" => {
            EXPECTED_DIMENSION_HASH.store(string_hash(&arguments.dimension_id), Ordering::Relaxed);
            EXPECTED_GAME_TIME.store(i64::from(arguments.game_time), Ordering::Relaxed);
            SAW_SAVE.store(false, Ordering::Relaxed);
            Ok("world save observer armed".to_owned())
        }
        "check" => {
            if SAW_SAVE.swap(false, Ordering::Relaxed) {
                Ok("guest received deferred world.save_observed notification".to_owned())
            } else {
                Err(failure(
                    "guest did not receive the expected world.save_observed notification",
                ))
            }
        }
        _ => Err(failure("operation must be either arm or check")),
    }
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
