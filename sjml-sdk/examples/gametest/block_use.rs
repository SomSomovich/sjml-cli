use sjml::{
    event,
    events::{self, BlockUseAction, EventPriority, ModuleBlockUse},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("block.use", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "block.use")]
fn use_registered_block(attempt: ModuleBlockUse) -> SjmlResult<BlockUseAction> {
    if attempt.block_id == "sjml_game_test:content/gametest_interactive_block"
        && attempt.face == "up"
        && attempt.hand == "main_hand"
        && attempt.item_id == "minecraft:diamond"
        && attempt.item_count == 3
    {
        Ok(BlockUseAction::ConsumeOne)
    } else {
        Ok(BlockUseAction::Pass)
    }
}
