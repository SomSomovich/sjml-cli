use sjml::world_explosion_knockback_attempt::{WorldExplosionKnockbackAttempt, EVENT_NAME};
use sjml::{
    event,
    events::{self, EventPriority},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe(EVENT_NAME, EventPriority::Normal)?;
    Ok(())
}

#[event(name = "world.explosion_knockback_attempt")]
fn veto_zombie_explosion_knockback(event: WorldExplosionKnockbackAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:zombie")
}
