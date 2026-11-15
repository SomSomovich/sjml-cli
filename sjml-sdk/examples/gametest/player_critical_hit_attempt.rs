use sjml::{
    event,
    events::{self, EventPriority},
    player_critical_hit_attempt::PlayerCriticalHitAttempt,
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("player.critical_hit_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.critical_hit_attempt")]
fn veto_horse_critical_hit(event: PlayerCriticalHitAttempt) -> SjmlResult<bool> {
    Ok(event.target_type_id == "minecraft:horse" && event.vanilla_critical)
}
