use sjml::world_sleep_finished_time::{WorldSleepFinishedTime, EVENT_NAME};
use sjml::{
    event,
    events::{self, EventPriority},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe(EVENT_NAME, EventPriority::Normal)?;
    Ok(())
}

#[event(name = "world.sleep_finished_time")]
fn observe_world_sleep_finished_time(event: WorldSleepFinishedTime) -> SjmlResult<()> {
    let _ = (event.dimension_id, event.new_time, event.current_time);
    Ok(())
}
