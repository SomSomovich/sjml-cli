use sjml::{
    event,
    events::{self, BlockPistonAttempt, EventPriority},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("block.piston_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "block.piston_attempt")]
fn veto_marked_piston(event: BlockPistonAttempt) -> SjmlResult<bool> {
    Ok(event.dimension_id == "minecraft:overworld"
        && event.direction == "east"
        && event.extending
        && event.block_id.contains("piston"))
}
