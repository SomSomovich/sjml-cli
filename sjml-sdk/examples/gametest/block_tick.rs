use sjml::{
    command, event,
    events::{self, BlockTick, EventPriority},
    world, BlockPos, ErrorCode, SjmlError, SjmlResult, StableDecode,
};

const BLOCK_ID: &str = "sjml_game_test:content/gametest_scheduled_block";

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("block.tick", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "block.tick")]
fn update_owned_scheduled_block(event: BlockTick) -> SjmlResult<()> {
    if event.block_id != BLOCK_ID
        || event.dimension_id != "minecraft:overworld"
        || event.property("ticked") != Some("false")
        || event.tick_count < 0
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "scheduled block tick callback received an unexpected detached snapshot",
        ));
    }
    let handle = world::get(&event.dimension_id)?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "scheduled block tick callback dimension is not loaded",
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

#[derive(StableDecode)]
#[sjml(version = 1)]
struct PositionArguments {
    x: i32,
    y: i32,
    z: i32,
}

#[command(name = "block.tick.schedule")]
fn schedule_registered_block_tick(arguments: PositionArguments) -> SjmlResult<String> {
    let handle = world::get("minecraft:overworld")?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the GameTest Overworld is not loaded",
        )
    })?;
    world::schedule_block_tick(
        handle,
        BlockPos {
            x: arguments.x,
            y: arguments.y,
            z: arguments.z,
        },
        BLOCK_ID,
        1,
    )?;
    Ok("scheduled one registered block tick".to_owned())
}
