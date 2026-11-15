use sjml::{
    event,
    events::{self, EventPriority},
    player_permissions_change_attempt::PlayerPermissionsChangeAttempt,
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("player.permissions_change_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.permissions_change_attempt")]
fn veto_operator_promotion(event: PlayerPermissionsChangeAttempt) -> SjmlResult<bool> {
    Ok(event.new_level == 4)
}
