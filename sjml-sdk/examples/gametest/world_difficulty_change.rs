use sjml::world_difficulty_change::{WorldDifficultyChange, EVENT_NAME};
use sjml::{
    event,
    events::{self, EventPriority},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe(EVENT_NAME, EventPriority::Normal)?;
    Ok(())
}

#[event(name = "world.difficulty_change")]
fn observe_world_difficulty_change(event: WorldDifficultyChange) -> SjmlResult<()> {
    let _ = (event.new_difficulty, event.old_difficulty);
    Ok(())
}
