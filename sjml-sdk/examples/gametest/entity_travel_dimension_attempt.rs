use sjml::{
    entity_travel_dimension_attempt::EntityTravelDimensionAttempt,
    event,
    events::{self, EventPriority},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.travel_dimension_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.travel_dimension_attempt")]
fn veto_overworld_to_nether(event: EntityTravelDimensionAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:pig"
        && event.from_dimension_id == "minecraft:overworld"
        && event.to_dimension_id == "minecraft:the_nether")
}
