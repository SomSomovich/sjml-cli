use sjml::prelude::*;
use sjml::world_chunk_data_save::{WorldChunkDataSave, WORLD_CHUNK_DATA_SAVE_EVENT_NAME};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe(WORLD_CHUNK_DATA_SAVE_EVENT_NAME, EventPriority::Normal)?;
    Ok(())
}

#[event(name = "world.chunk_data_save")]
fn observe_chunk_data_save(event: WorldChunkDataSave) -> SjmlResult<()> {
    host::log(
        LogLevel::Info,
        &format!(
            "saving {} chunk [{}, {}] with {} top-level NBT entries",
            event.dimension_id, event.chunk_x, event.chunk_z, event.data_entry_count
        ),
    )?;
    Ok(())
}
