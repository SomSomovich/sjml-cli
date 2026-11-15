use sjml::{
    entity_projectile_ammo_attempt::EntityProjectileAmmoAttempt,
    event,
    events::{self, EventPriority},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.projectile_ammo_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.projectile_ammo_attempt")]
fn veto_bow_ammo(event: EntityProjectileAmmoAttempt) -> SjmlResult<bool> {
    Ok(event.weapon.item_id == "minecraft:bow")
}
