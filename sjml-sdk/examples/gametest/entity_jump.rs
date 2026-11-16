use sjml::entity_jump::{EntityJump, EVENT_NAME};
use sjml::{
    event,
    events::{self, EventPriority},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe(EVENT_NAME, EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.jump")]
fn observe_entity_jump(event: EntityJump) -> SjmlResult<()> {
    let _ = (event.entity_id, event.entity_type_id, event.dimension_id);
    Ok(())
}
