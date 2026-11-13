use sjml::{command, entity, EntityHandle, ErrorCode, SjmlError, SjmlResult, StableDecode};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct EntityTargetArguments {
    mob_id: String,
    target_id: String,
}

#[command(name = "entity.target.check")]
fn read_set_and_clear_target(arguments: EntityTargetArguments) -> SjmlResult<String> {
    let mut mob: Option<EntityHandle> = None;
    let mut target: Option<EntityHandle> = None;
    for handle in entity::loaded()? {
        let snapshot = entity::snapshot(handle)?;
        if snapshot.uuid == arguments.mob_id {
            mob = Some(handle);
        }
        if snapshot.uuid == arguments.target_id {
            target = Some(handle);
        }
    }
    let mob = mob.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the target-control Mob is not loaded",
        )
    })?;
    let target = target.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the target-control entity is not loaded",
        )
    })?;
    let expected_target = entity::snapshot(target)?;

    if entity::target(mob)?.is_some() || entity::target_snapshot(mob)?.is_some() {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "the GameTest Mob must start without a current target",
        ));
    }
    entity::set_target(mob, target)?;
    if entity::target(mob)? != Some(target) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "the legacy target handle did not match the selected target",
        ));
    }
    let actual = entity::target_snapshot(mob)?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the Mob has no current non-player target after assignment",
        )
    })?;
    if actual.uuid != expected_target.uuid
        || actual.entity_type_id != expected_target.entity_type_id
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "the Mob did not retain the selected target UUID and type",
        ));
    }
    entity::clear_target(mob)?;
    if entity::target(mob)?.is_some() || entity::target_snapshot(mob)?.is_some() {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "the Mob target remained assigned after the clear operation",
        ));
    }
    Ok("guest read, selected, verified, and cleared a loaded Mob target".to_owned())
}
