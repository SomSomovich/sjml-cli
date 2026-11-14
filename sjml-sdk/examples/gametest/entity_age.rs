use sjml::{entity, ErrorCode, SjmlError, SjmlResult};

pub(super) fn reads_and_changes_a_loaded_pig_age(entity_id: &str) -> SjmlResult<String> {
    let handle = entity::loaded()?
        .into_iter()
        .find(|handle| {
            entity::snapshot(*handle).is_ok_and(|snapshot| {
                snapshot.uuid == entity_id && snapshot.entity_type_id == "minecraft:pig"
            })
        })
        .ok_or_else(|| {
            SjmlError::new(
                ErrorCode::InvalidArgument,
                "the age API GameTest pig is not loaded",
            )
        })?;

    let original_age = entity::age_ticks(handle)?;
    if original_age != -12_000 || !entity::is_baby(handle)? {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "the guest did not read the configured baby age",
        ));
    }

    entity::set_baby(handle, true)?;
    if entity::age_ticks(handle)? != entity::MIN_ENTITY_AGE_TICKS || !entity::is_baby(handle)? {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "setting baby state did not select a bounded negative age",
        ));
    }

    entity::set_baby(handle, false)?;
    if entity::age_ticks(handle)? != 0 || entity::is_baby(handle)? {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "setting adult state did not reset the age to zero",
        ));
    }

    entity::set_age_ticks(handle, 6_000)?;
    if entity::age_ticks(handle)? != 6_000 || entity::is_baby(handle)? {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "setting positive age did not preserve vanilla cooldown state",
        ));
    }
    entity::set_age_ticks(handle, original_age)?;
    Ok("guest read baby age, changed baby/adult state, and applied a positive cooldown".to_owned())
}
