use sjml::prelude::*;

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.shield_block_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.shield_block_attempt")]
fn veto_test_pig_shield_block(
    attempt: EntityShieldBlockAttempt,
) -> SjmlResult<EntityShieldBlockAttemptAction> {
    Ok(
        if attempt.entity_type_id == "minecraft:pig"
            && attempt.damage_type_id == "minecraft:generic"
        {
            EntityShieldBlockAttemptAction::Cancel
        } else {
            EntityShieldBlockAttemptAction::Pass
        },
    )
}
