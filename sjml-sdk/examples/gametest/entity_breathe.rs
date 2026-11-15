use sjml::{
    event,
    events::{self, EventPriority},
    SjmlResult,
};

#[path = "../../src/entity_breathe.rs"]
#[allow(dead_code)]
mod contract;
use contract::{EntityBreathe, EVENT_NAME};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe(EVENT_NAME, EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.breathe")]
fn observe_entity_breathe(event: EntityBreathe) -> SjmlResult<()> {
    let _ = (
        event.entity_id,
        event.entity_type_id,
        event.dimension_id,
        event.can_breathe,
        event.consume_air,
        event.refill_air,
    );
    Ok(())
}
