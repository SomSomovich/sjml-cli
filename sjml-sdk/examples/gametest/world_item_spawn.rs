use sjml::{command, world, ErrorCode, ItemSpawnVelocity, SjmlError, SjmlResult, StableDecode};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct ItemSpawnArguments {
    count: i32,
    x: i32,
    y: i32,
    z: i32,
    item_id: String,
}

#[command(name = "world.item.spawn")]
fn spawn_test_item(arguments: ItemSpawnArguments) -> SjmlResult<String> {
    let world = world::get("minecraft:overworld")?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the GameTest Overworld is unavailable",
        )
    })?;
    let count = u32::try_from(arguments.count)
        .map_err(|_| SjmlError::new(ErrorCode::InvalidArgument, "item count must be positive"))?;
    world::spawn_item(
        world,
        &arguments.item_id,
        count,
        world::RaycastPoint::new(
            f64::from(arguments.x) + 0.5,
            f64::from(arguments.y) + 0.5,
            f64::from(arguments.z) + 0.5,
        ),
        ItemSpawnVelocity::new(0.25, 1.0, -0.5),
    )?;
    Ok("ordinary item stack spawned".to_owned())
}
