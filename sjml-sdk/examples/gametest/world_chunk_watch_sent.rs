use sjml::prelude::*;
use sjml::world_chunk_watch_sent::{WorldChunkWatchSent, WORLD_CHUNK_WATCH_SENT_EVENT_NAME};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe(WORLD_CHUNK_WATCH_SENT_EVENT_NAME, EventPriority::Normal)?;
    Ok(())
}

#[event(name = "world.chunk_watch_sent")]
fn observe_chunk_watch_sent(event: WorldChunkWatchSent) -> SjmlResult<()> {
    host::log(
        LogLevel::Info,
        &format!(
            "player {} received {} [{}, {}]",
            event.player_id, event.dimension_id, event.chunk_x, event.chunk_z
        ),
    )?;
    Ok(())
}
