use sjml::player_flyable_fall::{PlayerFlyableFall, EVENT_NAME};
use sjml::{
    event,
    events::{self, EventPriority},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe(EVENT_NAME, EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.flyable_fall")]
fn observe_player_flyable_fall(event: PlayerFlyableFall) -> SjmlResult<()> {
    let _ = (
        event.player_id,
        event.dimension_id,
        event.distance,
        event.multiplier,
    );
    Ok(())
}
