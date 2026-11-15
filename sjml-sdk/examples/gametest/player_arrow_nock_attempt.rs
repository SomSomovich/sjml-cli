use sjml::{
    event,
    events::{self, EventPriority},
    player_arrow_nock_attempt::PlayerArrowNockAttempt,
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("player.arrow_nock_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.arrow_nock_attempt")]
fn veto_bow_draws(event: PlayerArrowNockAttempt) -> SjmlResult<bool> {
    Ok(event.item_id == "minecraft:bow")
}
