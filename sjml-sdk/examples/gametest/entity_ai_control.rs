use sjml::{command, entity, ErrorCode, SjmlError, SjmlResult, StableDecode};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct EntityAiControlArguments {
    entity_id: String,
}

#[command(name = "entity.ai_control.check")]
fn disable_selected_mob_ai(arguments: EntityAiControlArguments) -> SjmlResult<String> {
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
                "the entity AI-control GameTest pig is not loaded",
            )
        })?;

    if !entity::is_ai_enabled(target)? {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "the GameTest pig must start with AI enabled",
        ));
    }
    entity::set_ai_enabled(target, false)?;
    if entity::is_ai_enabled(target)? {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "the GameTest pig still reports AI enabled after the disable request",
        ));
    }
    Ok("guest disabled AI for the selected loaded pig".to_owned())
}
