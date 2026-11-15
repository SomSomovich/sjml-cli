use sjml::entity_finalize_spawn_attempt::EntityFinalizeSpawnAttempt;
use sjml::{
    event,
    events::{self, EventPriority},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.finalize_spawn_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.finalize_spawn_attempt")]
fn veto_husk_spawns(event: EntityFinalizeSpawnAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:husk")
}
