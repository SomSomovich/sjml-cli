use sjml::{
    event,
    events::{self, EventPriority, PlayerCommandAttempt},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("player.command_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.command_attempt")]
fn veto_marked_command(event: PlayerCommandAttempt) -> SjmlResult<bool> {
    Ok(event.command == "help"
        && !event.player_id.is_empty()
        && event.dimension_id == "minecraft:overworld")
}
