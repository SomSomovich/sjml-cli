use sjml::{
    event,
    events::{self, EventPriority},
    player_game_mode_change_attempt::PlayerGameModeChangeAttempt,
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("player.game_mode_change_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.game_mode_change_attempt")]
fn veto_creative_game_mode(event: PlayerGameModeChangeAttempt) -> SjmlResult<bool> {
    Ok(event.new_game_mode == "creative")
}
