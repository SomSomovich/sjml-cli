use sjml::player_wake_up::{PlayerWakeUp, EVENT_NAME};
use sjml::{
    event,
    events::{self, EventPriority},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe(EVENT_NAME, EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.wake_up")]
fn observe_player_wake_up(event: PlayerWakeUp) -> SjmlResult<()> {
    let _ = (
        event.player_id,
        event.dimension_id,
        event.wake_immediately,
        event.update_level,
    );
    Ok(())
}
