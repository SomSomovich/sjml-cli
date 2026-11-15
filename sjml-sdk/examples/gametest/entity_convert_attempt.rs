use sjml::{
    event,
    events::{self, EntityConvertAttempt, EventPriority},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.convert_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.convert_attempt")]
fn veto_marked_pig_conversion(event: EntityConvertAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:pig"
        && event.dimension_id == "minecraft:overworld"
        && event.outcome_entity_type_id == "minecraft:zombified_piglin")
}
