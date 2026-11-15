use sjml::{
    event,
    events::{self, EventPriority},
    player_grindstone_place_attempt::PlayerGrindstonePlaceAttempt,
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("player.grindstone_place_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.grindstone_place_attempt")]
fn veto_diamond_sword_placements(event: PlayerGrindstonePlaceAttempt) -> SjmlResult<bool> {
    Ok(event.top.item_id == "minecraft:diamond_sword")
}
