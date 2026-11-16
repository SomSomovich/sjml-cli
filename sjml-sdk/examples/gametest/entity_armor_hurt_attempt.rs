use sjml::entity_armor_hurt_attempt::EntityArmorHurtAttempt;
use sjml::{
    event,
    events::{self, EventPriority},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.armor_hurt_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.armor_hurt_attempt")]
fn veto_zombie_armor_damage(event: EntityArmorHurtAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:zombie")
}
