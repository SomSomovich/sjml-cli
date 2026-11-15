use sjml::{
    event,
    events::{self, EventPriority},
    EntityDestroyBlockAttempt, SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.destroy_block_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.destroy_block_attempt")]
fn veto_zombie_block_breaks(event: EntityDestroyBlockAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:zombie")
}
