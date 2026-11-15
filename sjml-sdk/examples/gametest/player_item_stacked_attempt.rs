use sjml::{
    event,
    events::{self, EventPriority},
    player_item_stacked_attempt::PlayerItemStackedAttempt,
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("player.item_stacked_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.item_stacked_attempt")]
fn veto_pickup_stacking(event: PlayerItemStackedAttempt) -> SjmlResult<bool> {
    Ok(event.click_action == "primary")
}
