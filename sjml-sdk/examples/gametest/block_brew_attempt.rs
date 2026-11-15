use sjml::{
    block_brew_attempt::BlockBrewAttempt,
    event,
    events::{self, EventPriority},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("block.brew_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "block.brew_attempt")]
fn veto_test_brewing_mix(event: BlockBrewAttempt) -> SjmlResult<bool> {
    let first_slot = event.stacks.first().map(|stack| stack.item_id.as_str());
    let last_slot = event.stacks.last().map(|stack| stack.item_id.as_str());
    Ok(first_slot == Some("minecraft:potion") && last_slot == Some("minecraft:nether_wart"))
}
