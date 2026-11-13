use sjml::{
    event,
    events::{self, EntityDropsAttempt, EventPriority},
    SjmlResult,
};

const VETOED_PIG_ID: &str = "00000000-0000-0000-0000-0000000000a1";

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.drops_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.drops_attempt")]
fn suppress_one_test_pig_drop(event: EntityDropsAttempt) -> SjmlResult<bool> {
    if event.entity_id != VETOED_PIG_ID || event.entity_type_id != "minecraft:pig" {
        return Ok(false);
    }

    let valid = event.dimension_id == "minecraft:overworld"
        && !event.recently_hit
        && !event.drops_truncated
        && event
            .drops
            .iter()
            .any(|drop| drop.item_id == "minecraft:porkchop" && (1..=3).contains(&drop.count));
    Ok(valid)
}
