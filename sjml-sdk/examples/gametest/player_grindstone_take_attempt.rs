use sjml::{
    event,
    events::{self, EventPriority},
    player_grindstone_take_attempt::PlayerGrindstoneTakeAttempt,
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("player.grindstone_take_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.grindstone_take_attempt")]
fn veto_diamond_sword_takes(event: PlayerGrindstoneTakeAttempt) -> SjmlResult<bool> {
    Ok(event.top.item_id == "minecraft:diamond_sword")
}
