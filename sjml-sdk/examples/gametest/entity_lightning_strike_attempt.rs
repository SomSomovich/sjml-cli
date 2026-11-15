use sjml::{
    event,
    events::{self, EntityLightningStrikeAttempt, EventPriority},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.lightning_strike_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.lightning_strike_attempt")]
fn veto_marked_pig_strike(event: EntityLightningStrikeAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:pig"
        && event.dimension_id == "minecraft:overworld"
        && !event.lightning_id.is_empty()
        && event.y.is_finite())
}
