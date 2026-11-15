use sjml::{
    event,
    events::{self, EntityTameAttempt, EventPriority},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.tame_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.tame_attempt")]
fn veto_wolf_taming(event: EntityTameAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:wolf"
        && event.dimension_id == "minecraft:overworld"
        && !event.tamer_id.is_empty()
        && !event.tamer_name.is_empty())
}
