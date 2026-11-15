use sjml::{
    event,
    events::{self, EventPriority, PlayerAnvilUpdateAttempt},
    SjmlResult,
};

const VETO_NAME: &str = "sjml-gametest-anvil-veto";

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("player.anvil_update_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.anvil_update_attempt")]
fn veto_marked_anvil_result(event: PlayerAnvilUpdateAttempt) -> SjmlResult<bool> {
    Ok(event.name.as_deref() == Some(VETO_NAME)
        && !event.player_id.is_empty()
        && event.left_count > 0
        && event.cost >= 0)
}
