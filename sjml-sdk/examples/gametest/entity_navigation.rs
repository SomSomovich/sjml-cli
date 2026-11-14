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
    let requested_target = BlockPos {
        x: arguments.x,
        y: arguments.y,
        z: arguments.z,
    };
    let status = entity::navigation_snapshot(target)?;
    if !status.active || status.target != Some(requested_target) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "the navigation status snapshot did not report the requested active pig path",
        ));
    }
    if !entity::stop_navigation(target)? {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "the guest could not confirm stopping its active navigation path",
        ));
    }
    let stopped = entity::navigation_snapshot(target)?;
    if stopped.active {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "the pig still reports an active path after stop_navigation",
        ));
    }
    Ok("guest started, inspected, and stopped a bounded path for the loaded pig".to_owned())
}
