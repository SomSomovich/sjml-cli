use std::sync::atomic::{AtomicU32, Ordering};

use sjml::{
    command, entity, event,
    events::{self, EventPriority, PlayerVillagerTraded},
    EntityHandle, ErrorCode, SjmlError, SjmlResult, StableDecode,
};

static EXPECTED_PLAYER: AtomicU32 = AtomicU32::new(0);
static EXPECTED_MERCHANT: AtomicU32 = AtomicU32::new(0);
static MATCHING_CALLBACKS: AtomicU32 = AtomicU32::new(0);
static SAW_VALID_SNAPSHOT: AtomicU32 = AtomicU32::new(0);
static SAW_VALID_LIVE_OFFER: AtomicU32 = AtomicU32::new(0);

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
struct VillagerTradeTestArguments {
    operation: String,
    player_id: String,
    merchant_id: String,
}

#[command(name = "player.villager_traded.test")]
fn run_villager_trade_test(arguments: VillagerTradeTestArguments) -> SjmlResult<String> {
    match arguments.operation.as_str() {
        "arm" => arm_villager_trade(&arguments),
        "check" => check_villager_trade(),
        _ => Err(failure("operation must be either arm or check")),
    }
}

fn arm_villager_trade(arguments: &VillagerTradeTestArguments) -> SjmlResult<String> {
    if arguments.player_id.len() != 36 || arguments.merchant_id.len() != 36 {
        return Err(failure("GameTest supplied a non-canonical entity UUID"));
    }
    EXPECTED_PLAYER.store(uuid_hash(&arguments.player_id), Ordering::Relaxed);
    EXPECTED_MERCHANT.store(uuid_hash(&arguments.merchant_id), Ordering::Relaxed);
    MATCHING_CALLBACKS.store(0, Ordering::Relaxed);
    SAW_VALID_SNAPSHOT.store(0, Ordering::Relaxed);
    SAW_VALID_LIVE_OFFER.store(0, Ordering::Relaxed);
    let merchant = find_merchant(&arguments.merchant_id)?;
    let offer_id = "sjml_game_test:content/gametest_librarian_trade";
    let before = entity::villager_offers(merchant)?;
    let initial = before
        .iter()
        .find(|offer| offer.id == offer_id && offer.registered)
        .ok_or_else(|| failure("registered librarian offer is missing from the live merchant"))?;
    if initial.input_a.item_id != "minecraft:emerald"
        || initial.input_a.count != 2
        || initial.current_input_a_count != 4
        || initial.input_b.is_some()
        || initial.result.item_id != "minecraft:diamond"
        || initial.result.count != 1
        || initial.uses != 0
        || initial.max_uses != 12
        || initial.demand != 20
        || initial.special_price != 0
    {
        return Err(failure(
            "registered librarian offer has an unexpected initial live snapshot",
        ));
    }

    entity::set_villager_offer_uses(merchant, offer_id, 3)?;
    entity::set_villager_offer_special_price(merchant, offer_id, -2)?;
    entity::update_villager_offer_demand(merchant, offer_id)?;
    let changed = entity::villager_offers(merchant)?;
    let changed = changed
        .iter()
        .find(|offer| offer.id == offer_id)
        .ok_or_else(|| failure("registered librarian offer disappeared after mutation"))?;
    if changed.uses != 3
        || changed.max_uses != 12
        || changed.demand != 14
        || changed.special_price != -2
        || changed.current_input_a_count != 1
    {
        return Err(failure(
            "live villager offer mutations did not match MerchantOffer semantics",
        ));
    }
    SAW_VALID_LIVE_OFFER.store(1, Ordering::Relaxed);
    Ok("villager trade observer armed".to_owned())
}

fn check_villager_trade() -> SjmlResult<String> {
    if MATCHING_CALLBACKS.swap(0, Ordering::Relaxed) == 1
        && SAW_VALID_SNAPSHOT.swap(0, Ordering::Relaxed) == 1
        && SAW_VALID_LIVE_OFFER.swap(0, Ordering::Relaxed) == 1
    {
        let merchant_id_hash = EXPECTED_MERCHANT.load(Ordering::Relaxed);
        let merchant = find_merchant_by_hash(merchant_id_hash)?;
        let offer_id = "sjml_game_test:content/gametest_librarian_trade";
        // Restore the original demand with a second vanilla update using a complementary stock
        // value: at uses=9 the update adds six, undoing the first update's minus six.
        entity::set_villager_offer_uses(merchant, offer_id, 9)?;
        entity::update_villager_offer_demand(merchant, offer_id)?;
        entity::set_villager_offer_uses(merchant, offer_id, 0)?;
        entity::set_villager_offer_special_price(merchant, offer_id, 0)?;
        let restored = entity::villager_offers(merchant)?;
        if !restored.iter().any(|offer| {
            offer.id == offer_id
                && offer.uses == 0
                && offer.max_uses == 12
                && offer.demand == 20
                && offer.special_price == 0
        }) {
            return Err(failure(
                "villager offer fixture did not restore its original economy state",
            ));
        }
        Ok("guest read and safely changed a registered live offer, then restored it".to_owned())
    } else {
        Err(failure(
            "player.villager_traded did not deliver one valid detached snapshot",
        ))
    }
}

fn find_merchant(uuid: &str) -> SjmlResult<EntityHandle> {
    find_merchant_by_hash(uuid_hash(uuid))
}

fn find_merchant_by_hash(expected: u32) -> SjmlResult<EntityHandle> {
    entity::loaded()?
        .into_iter()
        .find(|handle| {
            entity::snapshot(*handle).is_ok_and(|snapshot| uuid_hash(&snapshot.uuid) == expected)
        })
        .ok_or_else(|| failure("the GameTest merchant is not present in loaded entity handles"))
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
