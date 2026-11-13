use sjml::prelude::*;

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.teleport_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.teleport_attempt")]
fn veto_test_pig_teleport(
    attempt: EntityTeleportAttempt,
) -> SjmlResult<EntityTeleportAttemptAction> {
    Ok(
        if attempt.entity_type_id == "minecraft:pig" && attempt.target_x - attempt.from_x >= 64.0 {
            EntityTeleportAttemptAction::Cancel
        } else {
            EntityTeleportAttemptAction::Pass
        },
    )
}
