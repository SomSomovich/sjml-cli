use sjml::{command, entity, ErrorCode, SjmlError, SjmlResult, StableDecode};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct EntityTeleportArguments {
    entity_id: String,
}

#[command(name = "entity.teleport.check")]
fn checks_loaded_only_entity_teleport(arguments: EntityTeleportArguments) -> SjmlResult<String> {
    let handle = entity::loaded()?
        .into_iter()
        .find(|handle| {
            entity::snapshot(*handle).is_ok_and(|snapshot| {
                snapshot.uuid == arguments.entity_id && snapshot.entity_type_id == "minecraft:pig"
            })
        })
        .ok_or_else(|| {
            SjmlError::new(
                ErrorCode::InvalidArgument,
                "the entity teleport GameTest pig was not loaded",
            )
        })?;
    let original = entity::snapshot(handle)?;
    let temporary_x = original.x + 1.0;
    entity::teleport(handle, temporary_x, original.y, original.z)?;
    let moved = entity::snapshot(handle)?;
    if (moved.x - temporary_x).abs() > 0.0001
        || (moved.y - original.y).abs() > 0.0001
        || (moved.z - original.z).abs() > 0.0001
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "loaded same-dimension entity teleport did not reach the requested coordinates",
        ));
    }

    let unloaded = entity::teleport(handle, 1_000_000.0, original.y, original.z).unwrap_err();
    let after_rejection = entity::snapshot(handle)?;
    if unloaded.code() != ErrorCode::InvalidArgument
        || (after_rejection.x - temporary_x).abs() > 0.0001
        || (after_rejection.y - original.y).abs() > 0.0001
        || (after_rejection.z - original.z).abs() > 0.0001
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "teleport into an unloaded chunk must fail without moving the entity",
        ));
    }

    entity::teleport(handle, original.x, original.y, original.z)?;
    let restored = entity::snapshot(handle)?;
    if (restored.x - original.x).abs() > 0.0001
        || (restored.y - original.y).abs() > 0.0001
        || (restored.z - original.z).abs() > 0.0001
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity teleport GameTest did not restore the pig position",
        ));
    }
    Ok(
        "loaded entity teleport succeeded; unloaded target was rejected without movement"
            .to_owned(),
    )
}
