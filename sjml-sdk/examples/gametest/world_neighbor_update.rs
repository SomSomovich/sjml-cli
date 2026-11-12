use sjml::{command, world, BlockPos, ErrorCode, SjmlError, SjmlResult, StableDecode};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct NeighborUpdateCheck {
    x: i32,
    y: i32,
    z: i32,
    expect_unloaded: bool,
}

#[command(name = "world.neighbor.update.check")]
fn check_vanilla_neighbor_update(arguments: NeighborUpdateCheck) -> SjmlResult<String> {
    let overworld = world::get("minecraft:overworld")?
        .ok_or_else(|| SjmlError::new(ErrorCode::InvalidArgument, "the Overworld is not loaded"))?;
    let result = world::notify_neighbors_at(
        overworld,
        BlockPos {
            x: arguments.x,
            y: arguments.y,
            z: arguments.z,
        },
    );
    if arguments.expect_unloaded {
        return match result {
            Err(error) if error.code() == ErrorCode::InvalidArgument => {
                Ok("unloaded position was rejected without loading it".to_owned())
            }
            Err(error) => Err(error),
            Ok(()) => Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "unloaded position unexpectedly accepted a neighbor update",
            )),
        };
    }
    result?;
    Ok("vanilla neighbor update completed at the loaded block".to_owned())
}
