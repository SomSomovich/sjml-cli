use sjml::{
    event,
    events::{self, EventPriority},
    player_xp_pickup_attempt::PlayerXpPickupAttempt,
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("player.xp_pickup_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.xp_pickup_attempt")]
fn veto_marked_orb_pickup(event: PlayerXpPickupAttempt) -> SjmlResult<bool> {
    Ok(event.orb_value == 7)
}
