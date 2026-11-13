use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use sjml::{
    command, event,
    events::{self, EventPriority, PlayerEquipmentChanged, PlayerEquipmentSlot},
    ErrorCode, SjmlError, SjmlResult, StableDecode,
};

static EXPECTED_PLAYER: AtomicU32 = AtomicU32::new(0);
static SAW_EQUIPMENT_CHANGE: AtomicBool = AtomicBool::new(false);

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("player.equipment_changed", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.equipment_changed")]
fn observe_equipment_change(event: PlayerEquipmentChanged) -> SjmlResult<()> {
    let expected_player = EXPECTED_PLAYER.load(Ordering::Relaxed);
    let old_stack = event.previous.as_ref();
    let new_stack = event.current.as_ref();
    if expected_player != 0
        && uuid_hash(&event.player_id) == expected_player
        && event.slot == PlayerEquipmentSlot::MainHand
        && old_stack.is_some_and(|stack| {
            stack.item_id == "minecraft:iron_sword"
                && stack.count == 1
                && stack.damage == 11
                && stack.max_damage == 250
        })
        && new_stack.is_some_and(|stack| {
            stack.item_id == "minecraft:diamond_sword"
                && stack.count == 1
                && stack.damage == 7
                && stack.max_damage == 1_561
        })
    {
        SAW_EQUIPMENT_CHANGE.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct PlayerEquipmentChangedArguments {
    operation: String,
    player_id: String,
}

#[command(name = "player.equipment_changed")]
fn operate_equipment_change(arguments: PlayerEquipmentChangedArguments) -> SjmlResult<String> {
    if !canonical_uuid(&arguments.player_id) {
        return Err(failure("GameTest supplied a non-canonical player UUID"));
    }
    match arguments.operation.as_str() {
        "arm" => {
            EXPECTED_PLAYER.store(uuid_hash(&arguments.player_id), Ordering::Relaxed);
            SAW_EQUIPMENT_CHANGE.store(false, Ordering::Relaxed);
            Ok("player equipment-change observer armed".to_owned())
        }
        "check" => {
            let expected_player = EXPECTED_PLAYER.load(Ordering::Relaxed);
            if expected_player == 0
                || uuid_hash(&arguments.player_id) != expected_player
                || !SAW_EQUIPMENT_CHANGE.swap(false, Ordering::Relaxed)
            {
                return Err(failure(
                    "guest did not receive the expected server-player equipment transition",
                ));
            }
            Ok("guest received the expected player equipment transition".to_owned())
        }
        _ => Err(failure("operation must be either arm or check")),
    }
}

fn canonical_uuid(value: &str) -> bool {
    value.len() == 36
        && value.bytes().enumerate().all(|(index, byte)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                byte == b'-'
            } else {
                byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)
            }
        })
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
