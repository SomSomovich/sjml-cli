use std::sync::atomic::{AtomicU32, Ordering};

use sjml::{
    command, event,
    events::{self, EventPriority, PlayerVillagerTraded},
    ErrorCode, SjmlError, SjmlResult, StableDecode,
};

static EXPECTED_PLAYER: AtomicU32 = AtomicU32::new(0);
static EXPECTED_MERCHANT: AtomicU32 = AtomicU32::new(0);
static MATCHING_CALLBACKS: AtomicU32 = AtomicU32::new(0);
static SAW_VALID_SNAPSHOT: AtomicU32 = AtomicU32::new(0);

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("player.villager_traded", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.villager_traded")]
fn observe_test_villager_trade(event: PlayerVillagerTraded) -> SjmlResult<()> {
    if uuid_hash(&event.player_id) != EXPECTED_PLAYER.load(Ordering::Relaxed)
        || uuid_hash(&event.merchant_id) != EXPECTED_MERCHANT.load(Ordering::Relaxed)
        || EXPECTED_PLAYER.load(Ordering::Relaxed) == 0
        || EXPECTED_MERCHANT.load(Ordering::Relaxed) == 0
    {
        return Ok(());
    }
    MATCHING_CALLBACKS.fetch_add(1, Ordering::Relaxed);
    let valid = event.merchant_type_id == "minecraft:villager"
        && event.dimension_id == "minecraft:overworld"
        && event.profession_id == "minecraft:librarian"
        && event.profession_level == 2
        && event.cost_a_item_id == "minecraft:emerald"
        && event.cost_a_count == 4
        && event.cost_b_item_id.is_empty()
        && event.cost_b_count == 0
        && event.result_item_id == "minecraft:book"
        && event.result_count == 1
        && event.uses == 1
        && event.max_uses == 12;
    SAW_VALID_SNAPSHOT.store(u32::from(valid), Ordering::Relaxed);
    Ok(())
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct MerchantArguments {
    player_id: String,
    merchant_id: String,
}

#[command(name = "player.villager_traded.arm")]
fn arm_villager_trade(arguments: MerchantArguments) -> SjmlResult<String> {
    if arguments.player_id.len() != 36 || arguments.merchant_id.len() != 36 {
        return Err(failure("GameTest supplied a non-canonical entity UUID"));
    }
    EXPECTED_PLAYER.store(uuid_hash(&arguments.player_id), Ordering::Relaxed);
    EXPECTED_MERCHANT.store(uuid_hash(&arguments.merchant_id), Ordering::Relaxed);
    MATCHING_CALLBACKS.store(0, Ordering::Relaxed);
    SAW_VALID_SNAPSHOT.store(0, Ordering::Relaxed);
    Ok("villager trade observer armed".to_owned())
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct NoArguments {}

#[command(name = "player.villager_traded.check")]
fn check_villager_trade(_: NoArguments) -> SjmlResult<String> {
    if MATCHING_CALLBACKS.swap(0, Ordering::Relaxed) == 1
        && SAW_VALID_SNAPSHOT.swap(0, Ordering::Relaxed) == 1
    {
        Ok("guest received one bounded snapshot after the villager trade".to_owned())
    } else {
        Err(failure(
            "player.villager_traded did not deliver one valid detached snapshot",
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
