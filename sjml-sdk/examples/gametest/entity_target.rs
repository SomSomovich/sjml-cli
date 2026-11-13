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

    if entity::target(mob)?.is_some() {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "the GameTest Mob must start without a current target",
        ));
    }
    entity::set_target(mob, target)?;
    if entity::target(mob)? != Some(target) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "the guest did not read back the target handle it selected",
        ));
    }
    entity::clear_target(mob)?;
    if entity::target(mob)?.is_some() {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "the Mob target remained assigned after the clear operation",
        ));
    }
    Ok("guest read, selected, verified, and cleared a loaded Mob target".to_owned())
}
