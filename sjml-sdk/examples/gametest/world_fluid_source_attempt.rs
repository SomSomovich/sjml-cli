use sjml::world_fluid_source_attempt::{WorldFluidSourceAttempt, EVENT_NAME};
use sjml::{
    event,
    events::{self, EventPriority},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe(EVENT_NAME, EventPriority::Normal)?;
    Ok(())
}

#[event(name = "world.fluid_source_attempt")]
fn veto_water_source_formation(event: WorldFluidSourceAttempt) -> SjmlResult<bool> {
    Ok(event.fluid_id == "minecraft:water")
}
