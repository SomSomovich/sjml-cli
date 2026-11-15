use sjml::{
    entity_swap_items_attempt::EntitySwapItemsAttempt,
    event,
    events::{self, EventPriority},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.swap_items_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.swap_items_attempt")]
fn veto_zombie_hand_swap(event: EntitySwapItemsAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:zombie")
}
