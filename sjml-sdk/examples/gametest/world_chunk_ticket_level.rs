use sjml::prelude::*;
use sjml::world_chunk_ticket_level::{WorldChunkTicketLevel, WORLD_CHUNK_TICKET_LEVEL_EVENT_NAME};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe(WORLD_CHUNK_TICKET_LEVEL_EVENT_NAME, EventPriority::Normal)?;
    Ok(())
}

#[event(name = "world.chunk_ticket_level")]
fn observe_chunk_ticket_level(event: WorldChunkTicketLevel) -> SjmlResult<()> {
    host::log(
        LogLevel::Info,
        &format!(
            "{} [{}, {}] ticket {} -> {} (holder: {})",
            event.dimension_id,
            event.chunk_x,
            event.chunk_z,
            event.old_level,
            event.new_level,
            event.has_chunk_holder
        ),
    )?;
    Ok(())
}
