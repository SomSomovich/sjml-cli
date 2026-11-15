use sjml::{
    event,
    events::{self, EventPriority},
    PlayerSpawnPhantomsAttempt, SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("player.spawn_phantoms_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.spawn_phantoms_attempt")]
fn veto_large_phantom_spawns(event: PlayerSpawnPhantomsAttempt) -> SjmlResult<bool> {
    Ok(event.phantoms_to_spawn >= 3)
}
