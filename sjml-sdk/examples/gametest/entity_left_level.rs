use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering};

use sjml::{
    command, event,
    events::{self, EventPriority},
    ErrorCode, SjmlError, SjmlResult, StableDecode,
};

#[path = "../../src/entity_left_level.rs"]
#[allow(dead_code)]
mod contract;
use contract::EntityLeftLevel;

static EXPECTED_ENTITY_HASH: AtomicU32 = AtomicU32::new(0);
static EXPECTED_TYPE_HASH: AtomicU32 = AtomicU32::new(0);
static EXPECTED_DIMENSION_HASH: AtomicU32 = AtomicU32::new(0);
static EXPECTED_X: AtomicI32 = AtomicI32::new(0);
static EXPECTED_Y: AtomicI32 = AtomicI32::new(0);
static EXPECTED_Z: AtomicI32 = AtomicI32::new(0);
static SAW_ENTITY_LEFT_LEVEL: AtomicBool = AtomicBool::new(false);

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.left_level", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.left_level")]
fn observe_entity_left_level(event: EntityLeftLevel) -> SjmlResult<()> {
    if matches_expected(&event) {
        SAW_ENTITY_LEFT_LEVEL.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct EntityLeftLevelTestArguments {
    operation: String,
    entity_id: String,
    entity_type_id: String,
    dimension_id: String,
    x: i32,
    y: i32,
    z: i32,
}

#[command(name = "entity.left_level.test")]
fn test_entity_left_level(arguments: EntityLeftLevelTestArguments) -> SjmlResult<String> {
    match arguments.operation.as_str() {
        "arm" => {
            EXPECTED_ENTITY_HASH.store(string_hash(&arguments.entity_id), Ordering::Relaxed);
            EXPECTED_TYPE_HASH.store(string_hash(&arguments.entity_type_id), Ordering::Relaxed);
            EXPECTED_DIMENSION_HASH.store(string_hash(&arguments.dimension_id), Ordering::Relaxed);
            EXPECTED_X.store(arguments.x, Ordering::Relaxed);
            EXPECTED_Y.store(arguments.y, Ordering::Relaxed);
            EXPECTED_Z.store(arguments.z, Ordering::Relaxed);
            SAW_ENTITY_LEFT_LEVEL.store(false, Ordering::Relaxed);
            Ok("entity left-level observer armed".to_owned())
        }
        "check" => {
            if SAW_ENTITY_LEFT_LEVEL.swap(false, Ordering::Relaxed) {
                Ok("guest received detached entity.left_level snapshot".to_owned())
            } else {
                Err(failure(
                    "guest did not receive the expected entity.left_level snapshot",
                ))
            }
        }
        _ => Err(failure("operation must be either arm or check")),
    }
}

fn matches_expected(event: &EntityLeftLevel) -> bool {
    EXPECTED_ENTITY_HASH.load(Ordering::Relaxed) != 0
        && string_hash(&event.entity_id) == EXPECTED_ENTITY_HASH.load(Ordering::Relaxed)
        && string_hash(&event.entity_type_id) == EXPECTED_TYPE_HASH.load(Ordering::Relaxed)
        && string_hash(&event.dimension_id) == EXPECTED_DIMENSION_HASH.load(Ordering::Relaxed)
        && event.x == f64::from(EXPECTED_X.load(Ordering::Relaxed))
        && event.y == f64::from(EXPECTED_Y.load(Ordering::Relaxed))
        && event.z == f64::from(EXPECTED_Z.load(Ordering::Relaxed))
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
