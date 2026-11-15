use sjml::{
    event,
    events::{self, EventPriority, PlayerSpawnPointAttempt},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("player.spawn_point_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.spawn_point_attempt")]
fn veto_marked_spawn_point(event: PlayerSpawnPointAttempt) -> SjmlResult<bool> {
    Ok(!event.player_id.is_empty()
        && event.dimension_id == "minecraft:overworld"
        && event.new_spawn.is_some()
        && !event.forced)
}
