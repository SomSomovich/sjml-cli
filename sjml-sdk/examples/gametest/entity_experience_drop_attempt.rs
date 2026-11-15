use sjml::{
    event,
    events::{self, EntityExperienceDropAttempt, EventPriority},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.experience_drop_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.experience_drop_attempt")]
fn veto_marked_pig_experience(event: EntityExperienceDropAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:pig"
        && event.dimension_id == "minecraft:overworld"
        && event.original_experience == 7
        && event.dropped_experience == 7
        && event.attacking_player_id.is_some())
}
