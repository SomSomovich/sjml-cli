use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use sjml::{
    command, event,
    events::{self, EventPriority, PlayerAttackAttempt},
    ErrorCode, SjmlError, SjmlResult, StableDecode,
};

static ARMED: AtomicBool = AtomicBool::new(false);
static EXPECTED_PLAYER: AtomicU32 = AtomicU32::new(0);
static EXPECTED_TARGET: AtomicU32 = AtomicU32::new(0);
static MATCHING_CALLBACKS: AtomicU32 = AtomicU32::new(0);
static SAW_VALID_SNAPSHOT: AtomicBool = AtomicBool::new(false);

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("player.attack_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.attack_attempt")]
fn veto_armed_player_attack(event: PlayerAttackAttempt) -> SjmlResult<bool> {
    let expected_player = EXPECTED_PLAYER.load(Ordering::Relaxed);
    let expected_target = EXPECTED_TARGET.load(Ordering::Relaxed);
    if expected_player == 0
        || expected_target == 0
        || uuid_hash(&event.player_id) != expected_player
        || uuid_hash(&event.target_id) != expected_target
    {
        return Ok(false);
    }

    MATCHING_CALLBACKS.fetch_add(1, Ordering::Relaxed);
    let valid = event.player_id.len() == 36
        && event.target_id.len() == 36
        && event.player_id != event.target_id
        && event.target_type_id == "minecraft:pig"
        && event.dimension_id == "minecraft:overworld"
        && event.target_x.is_finite()
        && event.target_y.is_finite()
        && event.target_z.is_finite()
        && event.target_x.abs() <= 30_000_000.0
        && event.target_y.abs() <= 30_000_000.0
        && event.target_z.abs() <= 30_000_000.0;
    SAW_VALID_SNAPSHOT.store(valid, Ordering::Relaxed);
    Ok(valid && ARMED.swap(false, Ordering::Relaxed))
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct AttackArguments {
    player_id: String,
    target_id: String,
}

#[command(name = "player.attack_attempt.arm")]
fn arm_player_attack_attempt(arguments: AttackArguments) -> SjmlResult<String> {
    if arguments.player_id.len() != 36 || arguments.target_id.len() != 36 {
        return Err(failure("GameTest supplied a non-canonical entity UUID"));
    }
    EXPECTED_PLAYER.store(uuid_hash(&arguments.player_id), Ordering::Relaxed);
    EXPECTED_TARGET.store(uuid_hash(&arguments.target_id), Ordering::Relaxed);
    MATCHING_CALLBACKS.store(0, Ordering::Relaxed);
    SAW_VALID_SNAPSHOT.store(false, Ordering::Relaxed);
    ARMED.store(true, Ordering::Relaxed);
    Ok("player attack-attempt veto armed".to_owned())
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct NoArguments {}

#[command(name = "player.attack_attempt.check")]
fn check_player_attack_attempt(_: NoArguments) -> SjmlResult<String> {
    let callbacks = MATCHING_CALLBACKS.swap(0, Ordering::Relaxed);
    if callbacks == 1
        && !ARMED.load(Ordering::Relaxed)
        && SAW_VALID_SNAPSHOT.swap(false, Ordering::Relaxed)
    {
        Ok("guest received one detached player attack attempt and vetoed it".to_owned())
    } else {
        Err(failure(
            "player.attack_attempt did not deliver one valid callback and veto",
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
