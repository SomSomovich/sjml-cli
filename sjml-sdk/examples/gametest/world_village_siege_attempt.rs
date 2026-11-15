use sjml::world_village_siege_attempt::{WorldVillageSiegeAttempt, EVENT_NAME};
use sjml::{
    event,
    events::{self, EventPriority},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe(EVENT_NAME, EventPriority::Normal)?;
    Ok(())
}

#[event(name = "world.village_siege_attempt")]
fn veto_village_sieges(event: WorldVillageSiegeAttempt) -> SjmlResult<bool> {
    let _ = (
        event.dimension_id,
        event.player_id,
        event.x,
        event.y,
        event.z,
    );
    Ok(true)
}
