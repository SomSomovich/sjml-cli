use sjml::{
    event,
    events::{self, BlockRandomTick, EventPriority},
    world, ErrorCode, SjmlError, SjmlResult,
};

const BLOCK_ID: &str = "sjml_game_test:content/gametest_random_block";

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("block.random_tick", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "block.random_tick")]
fn update_owned_random_tick_block(event: BlockRandomTick) -> SjmlResult<()> {
    if event.block_id != BLOCK_ID
        || event.dimension_id != "minecraft:overworld"
        || event.property("ticked") != Some("false")
        || event.tick_count < 0
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "random block tick callback received an unexpected detached snapshot",
        ));
    }
    let handle = world::get(&event.dimension_id)?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "random block tick callback dimension is not loaded",
        )
    })?;
    world::set_block_properties(
        handle,
        event.position,
        BLOCK_ID,
        &[world::BlockProperty {
            name: "ticked".to_owned(),
            value: "true".to_owned(),
        }],
    )
}
