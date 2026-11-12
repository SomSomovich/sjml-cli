use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use sjml::{
    command, event,
    events::{self, EntityTick, EventPriority},
    ErrorCode, SjmlError, SjmlResult, StableDecode,
};

static EXPECTED_ENTITY: AtomicU32 = AtomicU32::new(0);
static SAW_EXPECTED_ENTITY_TICK: AtomicBool = AtomicBool::new(false);

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe_with_interval("entity.tick", EventPriority::Normal, 1)?;
    Ok(())
}

#[event(name = "entity.tick")]
fn observe_entity_tick(event: EntityTick) -> SjmlResult<()> {
    let expected = EXPECTED_ENTITY.load(Ordering::Relaxed);
    if expected != 0 && uuid_hash(&event.entity_id) == expected {
        SAW_EXPECTED_ENTITY_TICK.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct ArmArguments {
    entity_uuid: String,
}

#[command(name = "entity.tick.arm")]
fn arm_entity_tick(arguments: ArmArguments) -> SjmlResult<String> {
    if arguments.entity_uuid.len() != 36 {
        return Err(failure("GameTest supplied a non-canonical entity UUID"));
    }
    EXPECTED_ENTITY.store(uuid_hash(&arguments.entity_uuid), Ordering::Relaxed);
    SAW_EXPECTED_ENTITY_TICK.store(false, Ordering::Relaxed);
    Ok("entity tick observer armed".to_owned())
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct NoArguments {}

#[command(name = "entity.tick.check")]
fn check_entity_tick(_: NoArguments) -> SjmlResult<String> {
    if SAW_EXPECTED_ENTITY_TICK.swap(false, Ordering::Relaxed) {
        Ok("guest received the loaded entity tick".to_owned())
    } else {
        Err(failure(
            "the expected entity.tick callback was not delivered",
        ))
    }
}

fn uuid_hash(uuid: &str) -> u32 {
    uuid.bytes()
        .fold(2_166_136_261, |hash, byte| {
            (hash ^ u32::from(byte)).wrapping_mul(16_777_619)
        })
        .max(1)
}

fn failure(message: &'static str) -> SjmlError {
    SjmlError::new(ErrorCode::InvalidArgument, message)
}
