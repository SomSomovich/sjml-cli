use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use sjml::{
    command, event,
    events::{self, EventPriority, ModuleItemAttackAttempt, PlayerAttackAttempt},
    ErrorCode, SjmlError, SjmlResult, StableDecode,
};

static ARMED: AtomicBool = AtomicBool::new(false);
static EXPECTED_PLAYER: AtomicU32 = AtomicU32::new(0);
static EXPECTED_TARGET: AtomicU32 = AtomicU32::new(0);
static MATCHING_CALLBACKS: AtomicU32 = AtomicU32::new(0);
static SAW_VALID_SNAPSHOT: AtomicBool = AtomicBool::new(false);
static ITEM_TEST: AtomicBool = AtomicBool::new(false);
static ITEM_ARMED: AtomicBool = AtomicBool::new(false);
static ITEM_MATCHING_CALLBACKS: AtomicU32 = AtomicU32::new(0);
static SAW_VALID_ITEM_SNAPSHOT: AtomicBool = AtomicBool::new(false);

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("player.attack_attempt", EventPriority::Normal)?;
    events::subscribe("item.attack_attempt", EventPriority::Normal)?;
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
    let item_test = ITEM_TEST.load(Ordering::Relaxed);
    let expected_target_type = if item_test {
        "minecraft:cow"
    } else {
        "minecraft:pig"
    };
    let valid = event.player_id.len() == 36
        && event.target_id.len() == 36
        && event.player_id != event.target_id
        && event.target_type_id == expected_target_type
        && event.dimension_id == "minecraft:overworld"
        && event.target_x.is_finite()
        && event.target_y.is_finite()
        && event.target_z.is_finite()
        && event.target_x.abs() <= 30_000_000.0
        && event.target_y.abs() <= 30_000_000.0
        && event.target_z.abs() <= 30_000_000.0;
    SAW_VALID_SNAPSHOT.store(valid, Ordering::Relaxed);
    Ok(valid && !item_test && ARMED.swap(false, Ordering::Relaxed))
}

#[event(name = "item.attack_attempt")]
fn veto_registered_item_attack(event: ModuleItemAttackAttempt) -> SjmlResult<bool> {
    if !ITEM_TEST.load(Ordering::Relaxed)
        || uuid_hash(&event.player_id) != EXPECTED_PLAYER.load(Ordering::Relaxed)
        || uuid_hash(&event.target_id) != EXPECTED_TARGET.load(Ordering::Relaxed)
    {
        return Ok(false);
    }
    ITEM_MATCHING_CALLBACKS.fetch_add(1, Ordering::Relaxed);
    let valid = event.player_id.len() == 36
        && event.target_id.len() == 36
        && event.player_id != event.target_id
        && event.dimension_id == "minecraft:overworld"
        && event.item_id == "sjml_game_test:content/gametest_attack_item"
        && event.item_count == 1
        && event.target_type_id == "minecraft:cow"
        && event.target_x.is_finite()
        && event.target_y.is_finite()
        && event.target_z.is_finite();
    SAW_VALID_ITEM_SNAPSHOT.store(valid, Ordering::Relaxed);
    Ok(valid && ITEM_ARMED.swap(false, Ordering::Relaxed))
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct AttackArguments {
    player_id: String,
    target_id: String,
    test_case: String,
}

#[command(name = "player.attack_attempt.arm")]
fn arm_player_attack_attempt(arguments: AttackArguments) -> SjmlResult<String> {
    if arguments.player_id.len() != 36 || arguments.target_id.len() != 36 {
        return Err(failure("GameTest supplied a non-canonical entity UUID"));
    }
    let item_test = match arguments.test_case.as_str() {
        "player" => false,
        "item" => true,
        _ => return Err(failure("test_case must be player or item")),
    };
    ITEM_TEST.store(item_test, Ordering::Relaxed);
    EXPECTED_PLAYER.store(uuid_hash(&arguments.player_id), Ordering::Relaxed);
    EXPECTED_TARGET.store(uuid_hash(&arguments.target_id), Ordering::Relaxed);
    MATCHING_CALLBACKS.store(0, Ordering::Relaxed);
    ITEM_MATCHING_CALLBACKS.store(0, Ordering::Relaxed);
    SAW_VALID_SNAPSHOT.store(false, Ordering::Relaxed);
    SAW_VALID_ITEM_SNAPSHOT.store(false, Ordering::Relaxed);
    ARMED.store(!item_test, Ordering::Relaxed);
    ITEM_ARMED.store(item_test, Ordering::Relaxed);
    Ok("player attack-attempt veto armed".to_owned())
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct NoArguments {}

#[command(name = "player.attack_attempt.check")]
fn check_player_attack_attempt(_: NoArguments) -> SjmlResult<String> {
    let callbacks = MATCHING_CALLBACKS.swap(0, Ordering::Relaxed);
    let valid_general = callbacks == 1 && SAW_VALID_SNAPSHOT.swap(false, Ordering::Relaxed);
    if ITEM_TEST.swap(false, Ordering::Relaxed) {
        let item_callbacks = ITEM_MATCHING_CALLBACKS.swap(0, Ordering::Relaxed);
        if valid_general
            && !ARMED.load(Ordering::Relaxed)
            && item_callbacks == 1
            && !ITEM_ARMED.load(Ordering::Relaxed)
            && SAW_VALID_ITEM_SNAPSHOT.swap(false, Ordering::Relaxed)
        {
            Ok(
                "owner-scoped item attack callback received one snapshot and vetoed the attack"
                    .to_owned(),
            )
        } else {
            Err(failure(
                "item.attack_attempt did not deliver one valid owner-scoped veto",
            ))
        }
    } else if valid_general && !ARMED.load(Ordering::Relaxed) {
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
