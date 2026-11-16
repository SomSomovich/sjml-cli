use sjml::player_changed_dimension::{PlayerChangedDimension, EVENT_NAME};
use sjml::{
    event,
    events::{self, EventPriority},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe(EVENT_NAME, EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.changed_dimension")]
fn observe_player_changed_dimension(event: PlayerChangedDimension) -> SjmlResult<()> {
    let _ = (
        event.player_id,
        event.from_dimension_id,
        event.to_dimension_id,
    );
    Ok(())
}
