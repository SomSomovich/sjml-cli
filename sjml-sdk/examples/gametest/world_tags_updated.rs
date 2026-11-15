use sjml::world_tags_updated::{WorldTagsUpdated, EVENT_NAME};
use sjml::{
    event,
    events::{self, EventPriority},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe(EVENT_NAME, EventPriority::Normal)?;
    Ok(())
}

#[event(name = "world.tags_updated")]
fn observe_world_tags_updated(event: WorldTagsUpdated) -> SjmlResult<()> {
    let _ = (
        event.cause,
        event.from_client_packet,
        event.integrated_server_connection,
    );
    Ok(())
}
