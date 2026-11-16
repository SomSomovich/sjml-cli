use sjml::world_potential_spawns_attempt::{WorldPotentialSpawnsAttempt, EVENT_NAME};
use sjml::{
    event,
    events::{self, EventPriority},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe(EVENT_NAME, EventPriority::Normal)?;
    Ok(())
}

#[event(name = "world.potential_spawns_attempt")]
fn veto_monster_candidate_lists(event: WorldPotentialSpawnsAttempt) -> SjmlResult<bool> {
    let _ = (
        event.dimension_id,
        event.x,
        event.y,
        event.z,
        event.candidate_count,
    );
    Ok(event.category == "monster")
}
