use sjml::player_tracking::{
    PlayerStartTracking, PlayerStopTracking, PLAYER_START_TRACKING_EVENT_NAME,
    PLAYER_STOP_TRACKING_EVENT_NAME,
};
use sjml::prelude::*;

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe(PLAYER_START_TRACKING_EVENT_NAME, EventPriority::Normal)?;
    events::subscribe(PLAYER_STOP_TRACKING_EVENT_NAME, EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.start_tracking")]
fn observe_start_tracking(event: PlayerStartTracking) -> SjmlResult<()> {
    host::log(
        LogLevel::Info,
        &format!(
            "player {} in {} tracks {} ({})",
            event.player_id, event.dimension_id, event.target_id, event.target_type_id
        ),
    )?;
    Ok(())
}

#[event(name = "player.stop_tracking")]
fn observe_stop_tracking(event: PlayerStopTracking) -> SjmlResult<()> {
    host::log(
        LogLevel::Info,
        &format!(
            "player {} in {} stopped tracking {} ({})",
            event.player_id, event.dimension_id, event.target_id, event.target_type_id
        ),
    )?;
    Ok(())
}
