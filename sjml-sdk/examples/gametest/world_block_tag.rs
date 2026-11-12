use sjml::{command, inventory::BlockPos, world, SjmlError, SjmlResult, StableDecode};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct PositionArguments {
    x: i32,
    y: i32,
    z: i32,
}

#[command(name = "world.block_tag.check")]
fn check_loaded_block_tags(arguments: PositionArguments) -> SjmlResult<String> {
    let world = world::get("minecraft:overworld")?.ok_or_else(|| {
        SjmlError::new(
            sjml::ErrorCode::UnknownErrorCode,
            "the GameTest Overworld is unavailable",
        )
    })?;
    let oak = BlockPos {
        x: arguments.x,
        y: arguments.y,
        z: arguments.z,
    };
    let in_logs = world::block_has_tag(world, oak, "minecraft:logs")?;
    let stone = BlockPos {
        x: oak.x + 1,
        ..oak
    };
    let stone_in_logs = world::block_has_tag(world, stone, "minecraft:logs")?;
    let unloaded = world::block_has_tag(
        world,
        BlockPos {
            x: 26_000_000,
            y: oak.y,
            z: 26_000_000,
        },
        "minecraft:logs",
    )?;
    if in_logs != Some(true) || stone_in_logs != Some(false) || unloaded.is_some() {
        return Err(SjmlError::new(
            sjml::ErrorCode::InvalidArgument,
            "loaded block-tag lookup did not distinguish membership, misses, and unloaded chunks",
        ));
    }
    Ok(
        "loaded block tags matched the vanilla logs tag without loading the distant chunk"
            .to_owned(),
    )
}
