use sjml::{
    event,
    events::{self, EventPriority},
    player_arrow_loose_attempt::PlayerArrowLooseAttempt,
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("player.arrow_loose_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.arrow_loose_attempt")]
fn veto_marked_bow_release(event: PlayerArrowLooseAttempt) -> SjmlResult<bool> {
    Ok(event.item_id == "minecraft:bow" && event.charge == 20)
}
