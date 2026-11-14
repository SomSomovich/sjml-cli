use sjml::{
    command, world, BlockPos, BlockUpdateOptions, ErrorCode, SjmlError, SjmlResult, StableDecode,
};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct SetBlockOptionsCheck {
    x: i32,
    y: i32,
    z: i32,
    redstone: bool,
    update_neighbors: bool,
    update_clients: bool,
    known_shape: bool,
    suppress_drops: bool,
    replace_with_air: bool,
    expect_unloaded: bool,
}

#[command(name = "world.block.update.options.check")]
fn check_block_update_options(arguments: SetBlockOptionsCheck) -> SjmlResult<String> {
    let overworld = world::get("minecraft:overworld")?
        .ok_or_else(|| SjmlError::new(ErrorCode::InvalidArgument, "the Overworld is not loaded"))?;
    let result = world::set_block_with_options(
        overworld,
        BlockPos {
            x: arguments.x,
            y: arguments.y,
            z: arguments.z,
        },
        if arguments.replace_with_air {
            "minecraft:air"
        } else if arguments.redstone {
            "minecraft:redstone_block"
        } else {
            "minecraft:stone"
        },
        BlockUpdateOptions {
            update_neighbors: arguments.update_neighbors,
            update_clients: arguments.update_clients,
            known_shape: arguments.known_shape,
            suppress_drops: arguments.suppress_drops,
        },
    );
    if arguments.expect_unloaded {
        return match result {
            Err(error) if error.code() == ErrorCode::InvalidArgument => {
                Ok("unloaded block position was rejected without loading it".to_owned())
            }
            Err(error) => Err(error),
            Ok(()) => Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "unloaded position unexpectedly accepted a block update",
            )),
        };
    }
    result?;
    Ok("block replacement accepted the requested safe update options".to_owned())
}
