use sjml::{
    event,
    events::{self, BlockDropsAttempt, EventPriority},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("block.drops_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "block.drops_attempt")]
fn suppress_test_diamond_ore_drops(event: BlockDropsAttempt) -> SjmlResult<bool> {
    if event.block_id != "minecraft:diamond_ore" {
        return Ok(false);
    }

    let valid = event.dimension_id == "minecraft:overworld"
        && event.breaker.as_ref().is_some_and(|breaker| {
            !breaker.entity_id.is_empty() && breaker.entity_type_id == "minecraft:player"
        })
        && event.tool.as_ref().is_some_and(|tool| {
            tool.item_id == "minecraft:iron_pickaxe"
                && tool.count == 1
                && tool.damage.is_some()
                && tool.max_damage.is_some_and(|maximum| maximum > 0)
        })
        && event
            .drops
            .iter()
            .any(|drop| drop.item_id == "minecraft:diamond" && drop.count > 0)
        && event.experience > 0
        && !event.drops_truncated;

    Ok(valid)
}
