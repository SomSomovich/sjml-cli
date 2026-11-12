use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use sjml::{
    command, event,
    events::{self, EventPriority, PlayerFishedAttempt},
    ErrorCode, SjmlError, SjmlResult, StableDecode,
};

static ARMED: AtomicBool = AtomicBool::new(false);
static EXPECTED_PLAYER: AtomicU32 = AtomicU32::new(0);
static MATCHING_CALLBACKS: AtomicU32 = AtomicU32::new(0);
static SAW_VALID_SNAPSHOT: AtomicBool = AtomicBool::new(false);

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("player.fished_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.fished_attempt")]
fn veto_armed_fishing_catch(event: PlayerFishedAttempt) -> SjmlResult<bool> {
    if uuid_hash(&event.player_id) != EXPECTED_PLAYER.load(Ordering::Relaxed)
        || EXPECTED_PLAYER.load(Ordering::Relaxed) == 0
    {
        return Ok(false);
    }
    MATCHING_CALLBACKS.fetch_add(1, Ordering::Relaxed);
    let valid = event.player_id.len() == 36
        && event.dimension_id == "minecraft:overworld"
        && event.hook_x == 4.25
        && event.hook_y == 65.0
        && event.hook_z == -2.5
        && event.rod_damage == 3
        && !event.drops_truncated
        && event.drops.len() == 2
        && event.drops[0].item_id == "minecraft:cod"
        && event.drops[0].count == 2
        && event.drops[1].item_id == "minecraft:bone"
        && event.drops[1].count == 1;
    SAW_VALID_SNAPSHOT.store(valid, Ordering::Relaxed);
    Ok(valid && ARMED.swap(false, Ordering::Relaxed))
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct PlayerArguments {
    player_id: String,
}

#[command(name = "player.fished_attempt.arm")]
fn arm_fishing_catch(arguments: PlayerArguments) -> SjmlResult<String> {
    if arguments.player_id.len() != 36 {
        return Err(failure("GameTest supplied a non-canonical player UUID"));
    }
    EXPECTED_PLAYER.store(uuid_hash(&arguments.player_id), Ordering::Relaxed);
    MATCHING_CALLBACKS.store(0, Ordering::Relaxed);
    SAW_VALID_SNAPSHOT.store(false, Ordering::Relaxed);
    ARMED.store(true, Ordering::Relaxed);
    Ok("fishing catch veto armed".to_owned())
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct NoArguments {}

#[command(name = "player.fished_attempt.check")]
fn check_fishing_catch_veto(_: NoArguments) -> SjmlResult<String> {
    let callbacks = MATCHING_CALLBACKS.swap(0, Ordering::Relaxed);
    if callbacks == 1
        && !ARMED.load(Ordering::Relaxed)
        && SAW_VALID_SNAPSHOT.swap(false, Ordering::Relaxed)
    {
        Ok("guest received one bounded fishing snapshot and vetoed its catch".to_owned())
    } else {
        Err(failure(
            "player.fished_attempt did not deliver one valid callback and veto",
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
