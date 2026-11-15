use sjml::{
    event,
    events::{self, EventPriority},
    player_sweep_attack_attempt::PlayerSweepAttackAttempt,
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("player.sweep_attack_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.sweep_attack_attempt")]
fn veto_horse_sweeps(event: PlayerSweepAttackAttempt) -> SjmlResult<bool> {
    Ok(event.target_type_id == "minecraft:horse" && event.vanilla_sweep)
}
