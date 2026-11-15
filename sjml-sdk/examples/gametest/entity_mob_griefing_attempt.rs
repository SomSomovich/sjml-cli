use sjml::{
    entity_mob_griefing_attempt::EntityMobGriefingAttempt,
    event,
    events::{self, EventPriority},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.mob_griefing_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.mob_griefing_attempt")]
fn veto_zombie_griefing(event: EntityMobGriefingAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:zombie")
}
