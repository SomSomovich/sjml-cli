use sjml::{command, world, BlockPos, ErrorCode, SjmlError, SjmlResult, StableDecode};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct DestroyArguments {
    x: i32,
    y: i32,
    z: i32,
    drop_items: bool,
}

#[command(name = "world.block.destroy")]
fn destroy_block(arguments: DestroyArguments) -> SjmlResult<String> {
    let world = world::get("minecraft:overworld")?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the overworld must already be loaded",
        )
    })?;
    world::destroy_block(
        world,
        BlockPos {
            x: arguments.x,
            y: arguments.y,
            z: arguments.z,
        },
        arguments.drop_items,
    )?;
    Ok("guest block destroyed".to_owned())
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct BlockStateArguments {
    x: i32,
    y: i32,
    z: i32,
    state: String,
}

#[command(name = "world.block_state.set")]
fn set_block_state(arguments: BlockStateArguments) -> SjmlResult<String> {
    let world = world::get("minecraft:overworld")?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the overworld must already be loaded",
        )
    })?;
    world::set_block_state(
        world,
        BlockPos {
            x: arguments.x,
            y: arguments.y,
            z: arguments.z,
        },
        &arguments.state,
    )?;
    Ok("guest block state applied".to_owned())
}
