use sjml::{
    event,
    events::{self, EntityMobSplitAttempt, EventPriority},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.mob_split_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.mob_split_attempt")]
fn veto_slime_splits(event: EntityMobSplitAttempt) -> SjmlResult<bool> {
    Ok(event.parent_type_id == "minecraft:slime"
        && event.dimension_id == "minecraft:overworld"
        && event.child_count > 0)
}
