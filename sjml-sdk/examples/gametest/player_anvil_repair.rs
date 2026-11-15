use sjml::{
    event,
    events::{self, EventPriority},
    player_anvil_repair::PlayerAnvilRepair,
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("player.anvil_repair", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.anvil_repair")]
fn observe_anvil_repair(event: PlayerAnvilRepair) -> SjmlResult<()> {
    let _tracked = event.break_chance > 0.0
        && event.output.count > 0
        && event.left.item_id == "minecraft:diamond_sword"
        && event.right.item_id == "minecraft:diamond";
    Ok(())
}
