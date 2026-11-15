use sjml::{
    event,
    events::{self, EventPriority},
    ItemExpireAttempt, SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("item.expire_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "item.expire_attempt")]
fn keep_dropped_diamonds(event: ItemExpireAttempt) -> SjmlResult<bool> {
    Ok(event.item_id == "minecraft:diamond")
}
