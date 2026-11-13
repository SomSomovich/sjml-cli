use sjml::prelude::{AttributeModifierOperation, LivingAttribute};
use sjml::{command, entity, entity_attributes, ErrorCode, SjmlError, SjmlResult};

#[command(name = "entity.attribute_modifier.check")]
fn check_transient_modifier(arguments: super::EntityArguments) -> SjmlResult<String> {
    let handle = entity::loaded()?
        .into_iter()
        .find(|handle| {
            entity::snapshot(*handle).is_ok_and(|snapshot| snapshot.uuid == arguments.entity_id)
        })
        .ok_or_else(|| {
            SjmlError::new(
                ErrorCode::InvalidArgument,
                "the entity attribute modifier GameTest pig is missing",
            )
        })?;
    let attribute = LivingAttribute::MovementSpeed;
    let before = entity_attributes::read(handle, attribute)?;
    let key = "gametest_speed_bonus";

    entity_attributes::add_transient_modifier(
        handle,
        attribute,
        key,
        0.5,
        AttributeModifierOperation::AddValue,
    )?;
    let during = entity_attributes::read(handle, attribute);
    let removed = entity_attributes::remove_transient_modifier(handle, key);
    let after = entity_attributes::read(handle, attribute);

    let during = during?;
    let removed = removed?;
    let after = after?;
    if (during.value - (before.value + 0.5)).abs() > 0.000_001 {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "the transient entity attribute modifier did not affect the live value",
        ));
    }
    if !removed || (after.value - before.value).abs() > 0.000_001 {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "the transient entity attribute modifier was not removed from the live value",
        ));
    }
    Ok("transient entity attribute modifier applied and removed".to_owned())
}
