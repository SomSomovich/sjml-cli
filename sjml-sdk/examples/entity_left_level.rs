#![cfg_attr(
    not(all(target_arch = "wasm32", target_os = "unknown")),
    allow(dead_code)
)]

#[path = "../src/entity_left_level.rs"]
#[allow(dead_code)]
mod entity_left_level;

use entity_left_level::{EntityLeftLevel, ENTITY_LEFT_LEVEL_EVENT_NAME};
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe(ENTITY_LEFT_LEVEL_EVENT_NAME, EventPriority::Normal)?;
    Ok(())
}

/// Entity-leave events are observational and can use async handlers to keep removal hooks prompt.
#[event_async(name = "entity.left_level")]
async fn entity_left_level(event: EntityLeftLevel) -> SjmlResult<()> {
    host::log(
        LogLevel::Info,
        &format!(
            "entity {} ({}) left {} at [{}, {}, {}]",
            event.entity_id, event.entity_type_id, event.dimension_id, event.x, event.y, event.z,
        ),
    )?;
    Ok(())
}
