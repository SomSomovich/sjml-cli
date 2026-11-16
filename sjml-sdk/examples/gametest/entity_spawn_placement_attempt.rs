use sjml::entity_spawn_placement_attempt::EntitySpawnPlacementAttempt;
use sjml::{
    event,
    events::{self, EventPriority},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.spawn_placement_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.spawn_placement_attempt")]
fn deny_zombie_placements(event: EntitySpawnPlacementAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:zombie")
}
