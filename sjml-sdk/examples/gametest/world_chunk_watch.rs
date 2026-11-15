use sjml::prelude::*;
use sjml::world_chunk_watch::{
    WorldChunkUnwatched, WorldChunkWatched, WORLD_CHUNK_UNWATCH_EVENT_NAME,
    WORLD_CHUNK_WATCH_EVENT_NAME,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe(WORLD_CHUNK_WATCH_EVENT_NAME, EventPriority::Normal)?;
    events::subscribe(WORLD_CHUNK_UNWATCH_EVENT_NAME, EventPriority::Normal)?;
    Ok(())
}

#[event(name = "world.chunk_watch")]
fn observe_chunk_watch(event: WorldChunkWatched) -> SjmlResult<()> {
    host::log(
        LogLevel::Info,
        &format!(
            "player {} watches {} [{}, {}]",
            event.player_id, event.dimension_id, event.chunk_x, event.chunk_z
        ),
    )?;
    Ok(())
}

#[event(name = "world.chunk_unwatch")]
fn observe_chunk_unwatch(event: WorldChunkUnwatched) -> SjmlResult<()> {
    host::log(
        LogLevel::Info,
        &format!(
            "player {} stopped watching {} [{}, {}]",
            event.player_id, event.dimension_id, event.chunk_x, event.chunk_z
        ),
    )?;
    Ok(())
}
