use sjml::entity_drown_attempt::EntityDrownAttempt;
use sjml::{
    event,
    events::{self, EventPriority},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.drown_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.drown_attempt")]
fn veto_zombie_drowning(event: EntityDrownAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:zombie")
}
