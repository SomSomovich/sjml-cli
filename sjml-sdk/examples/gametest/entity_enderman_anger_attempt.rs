use sjml::{
    event,
    events::{self, EventPriority},
    EntityEndermanAngerAttempt, SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.enderman_anger_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.enderman_anger_attempt")]
fn veto_enderman_anger(event: EntityEndermanAngerAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:enderman")
}
