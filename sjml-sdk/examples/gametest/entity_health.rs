use sjml::{command, entity, ErrorCode, SjmlError, SjmlResult, StableDecode};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct EntityHealthArguments {
    entity_id: String,
}

#[command(name = "entity.health.check")]
fn reads_heals_and_rereads(arguments: EntityHealthArguments) -> SjmlResult<String> {
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
                "the entity health GameTest pig is not loaded",
            )
        })?;

    let before = entity::vitals(handle)?;
    let amount = (before.max_health - before.health).min(2.0);
    if amount <= 0.0 {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "the entity health GameTest pig must be injured before healing",
        ));
    }
    entity::heal(handle, amount)?;
    let after = entity::vitals(handle)?;
    if (after.health - before.health - amount).abs() > 0.001 {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "the ordinary Minecraft heal path did not apply the expected amount",
        ));
    }
    Ok("guest read vitals, healed the pig, and reread vitals".to_owned())
}
