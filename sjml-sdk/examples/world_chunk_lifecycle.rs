#![cfg_attr(
    not(all(target_arch = "wasm32", target_os = "unknown")),
    allow(dead_code)
)]

#[path = "../src/world_chunk_lifecycle.rs"]
#[allow(dead_code)]
mod world_chunk_lifecycle;

use sjml::prelude::*;
use world_chunk_lifecycle::{
    WorldChunkLifecycle, WORLD_CHUNK_LOADED_EVENT_NAME, WORLD_CHUNK_UNLOADED_EVENT_NAME,
};

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe(WORLD_CHUNK_LOADED_EVENT_NAME, EventPriority::Normal)?;
    events::subscribe(WORLD_CHUNK_UNLOADED_EVENT_NAME, EventPriority::Normal)?;
    Ok(())
}

/// Chunk lifecycle hooks can fire frequently and while Minecraft is changing chunk state.
/// Async handlers enqueue work promptly; this payload remains detached and contains no chunk API.
#[event_async(name = "world.chunk_loaded")]
async fn chunk_loaded(event: WorldChunkLifecycle) -> SjmlResult<()> {
    host::log(
        LogLevel::Info,
        &format!(
            "chunk loaded: {} [{}, {}]",
            event.dimension_id, event.chunk_x, event.chunk_z
        ),
    )?;
    Ok(())
}

#[event_async(name = "world.chunk_unloaded")]
async fn chunk_unloaded(event: WorldChunkLifecycle) -> SjmlResult<()> {
    host::log(
        LogLevel::Info,
        &format!(
            "chunk unloaded: {} [{}, {}]",
            event.dimension_id, event.chunk_x, event.chunk_z
        ),
    )?;
    Ok(())
}
