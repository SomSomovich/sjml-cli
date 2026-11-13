use sjml::{command, entity, ErrorCode, SjmlError, SjmlResult, StableDecode};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct EntityRotationArguments {
    entity_id: String,
}

#[command(name = "entity.rotation.check")]
fn reads_changes_and_restores_rotation(arguments: EntityRotationArguments) -> SjmlResult<String> {
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
                "the entity rotation GameTest pig is not loaded",
            )
        })?;

    let original = entity::rotation(handle)?;
    let replacement = entity::EntityRotation::new(123.25, -22.5)?;
    entity::set_rotation(handle, replacement)?;
    let observed = entity::rotation(handle);
    let restored = entity::set_rotation(handle, original);
    let observed = observed?;
    restored?;

    if observed != replacement {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "the guest did not observe the temporary entity rotation",
        ));
    }
    Ok("guest read, changed, reread, and restored the pig rotation".to_owned())
}
