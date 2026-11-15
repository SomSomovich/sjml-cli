use sjml::{
    event,
    events::{self, EventPriority},
    EntityDespawnAttempt, SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.despawn_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.despawn_attempt")]
fn veto_stray_despawn(event: EntityDespawnAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:stray")
}
