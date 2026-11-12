use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use sjml::{
    command, event,
    events::{self, EntityBreedAttempt, EventPriority},
    ErrorCode, SjmlError, SjmlResult, StableDecode,
};

static ARMED: AtomicBool = AtomicBool::new(false);
static EXPECTED_PARENT_A: AtomicU32 = AtomicU32::new(0);
static EXPECTED_PARENT_B: AtomicU32 = AtomicU32::new(0);
static MATCHING_CALLBACKS: AtomicU32 = AtomicU32::new(0);
static SAW_VALID_SNAPSHOT: AtomicBool = AtomicBool::new(false);

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.breed_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.breed_attempt")]
fn veto_armed_breeding(event: EntityBreedAttempt) -> SjmlResult<bool> {
    let parent_a = uuid_hash(&event.parent_a_id);
    let parent_b = uuid_hash(&event.parent_b_id);
    let expected_a = EXPECTED_PARENT_A.load(Ordering::Relaxed);
    let expected_b = EXPECTED_PARENT_B.load(Ordering::Relaxed);
    let matches_expected_parents = (parent_a == expected_a && parent_b == expected_b)
        || (parent_a == expected_b && parent_b == expected_a);
    if !matches_expected_parents || expected_a == 0 || expected_b == 0 {
        return Ok(false);
    }

    MATCHING_CALLBACKS.fetch_add(1, Ordering::Relaxed);
    let valid = event.parent_a_id.len() == 36
        && event.parent_b_id.len() == 36
        && event.parent_a_id != event.parent_b_id
        && event.parent_a_type_id == "minecraft:cow"
        && event.parent_b_type_id == "minecraft:cow"
        && event.child_id.len() == 36
        && event.child_id != event.parent_a_id
        && event.child_id != event.parent_b_id
        && event.child_type_id == "minecraft:cow"
        && event.dimension_id == "minecraft:overworld";
    SAW_VALID_SNAPSHOT.store(valid, Ordering::Relaxed);
    Ok(valid && ARMED.swap(false, Ordering::Relaxed))
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct ParentArguments {
    parent_a_id: String,
    parent_b_id: String,
}

#[command(name = "entity.breed_attempt.arm")]
fn arm_breed_attempt(arguments: ParentArguments) -> SjmlResult<String> {
    if arguments.parent_a_id.len() != 36 || arguments.parent_b_id.len() != 36 {
        return Err(failure("GameTest supplied a non-canonical parent UUID"));
    }
    EXPECTED_PARENT_A.store(uuid_hash(&arguments.parent_a_id), Ordering::Relaxed);
    EXPECTED_PARENT_B.store(uuid_hash(&arguments.parent_b_id), Ordering::Relaxed);
    MATCHING_CALLBACKS.store(0, Ordering::Relaxed);
    SAW_VALID_SNAPSHOT.store(false, Ordering::Relaxed);
    ARMED.store(true, Ordering::Relaxed);
    Ok("entity breed-attempt veto armed".to_owned())
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct NoArguments {}

#[command(name = "entity.breed_attempt.check")]
fn check_breed_attempt(_: NoArguments) -> SjmlResult<String> {
    let callbacks = MATCHING_CALLBACKS.swap(0, Ordering::Relaxed);
    if callbacks == 1
        && !ARMED.load(Ordering::Relaxed)
        && SAW_VALID_SNAPSHOT.swap(false, Ordering::Relaxed)
    {
        Ok("guest received one detached breeding attempt and vetoed the child".to_owned())
    } else {
        Err(failure(
            "entity.breed_attempt did not deliver one valid callback and veto",
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
