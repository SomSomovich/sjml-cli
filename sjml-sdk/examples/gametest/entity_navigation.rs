use sjml::{command, entity, inventory::BlockPos, ErrorCode, SjmlError, SjmlResult, StableDecode};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct EntityNavigationArguments {
    entity_id: String,
    x: i32,
    y: i32,
    z: i32,
}

#[command(name = "entity.navigation.check")]
fn starts_bounded_navigation(arguments: EntityNavigationArguments) -> SjmlResult<String> {
    let target = entity::loaded()?
        .into_iter()
        .find(|handle| {
            entity::snapshot(*handle).is_ok_and(|snapshot| {
                snapshot.uuid == arguments.entity_id && snapshot.entity_type_id == "minecraft:pig"
            })
        })
        .ok_or_else(|| {
            SjmlError::new(
                ErrorCode::InvalidArgument,
                "the entity navigation GameTest pig is not loaded",
            )
        })?;

    if !entity::navigate_to(
        target,
        BlockPos {
            x: arguments.x,
            y: arguments.y,
            z: arguments.z,
        },
        1.0,
    )? {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "Minecraft did not accept a navigation path for the GameTest pig",
        ));
    }
    Ok("guest started a bounded path for the loaded pig".to_owned())
}
