use sjml::{command, entity, EntityEquipmentSlot, ErrorCode, SjmlError, SjmlResult, StableDecode};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct EntityEquipmentArguments {
    entity_id: String,
}

#[command(name = "entity.equipment.check")]
fn reads_only_the_expected_loaded_mob_equipment(
    arguments: EntityEquipmentArguments,
) -> SjmlResult<String> {
    let handle = entity::loaded()?
        .into_iter()
        .find(|handle| {
            entity::snapshot(*handle).is_ok_and(|snapshot| {
                snapshot.uuid == arguments.entity_id
                    && snapshot.entity_type_id == "minecraft:zombie"
            })
        })
        .ok_or_else(|| {
            SjmlError::new(
                ErrorCode::InvalidArgument,
                "the equipment GameTest zombie is not loaded",
            )
        })?;

    let equipment = entity::equipment(handle)?;
    let sword = equipment.main_hand.as_ref().ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the zombie main-hand slot must be occupied",
        )
    })?;
    let helmet = equipment.head.as_ref().ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the zombie head slot must be occupied",
        )
    })?;
    let leggings = equipment.legs.as_ref().ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the zombie legs slot must be occupied",
        )
    })?;
    let boots = equipment.feet.as_ref().ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the zombie feet slot must be occupied",
        )
    })?;
    if sword.item_id != "minecraft:iron_sword"
        || sword.count != 1
        || sword.damage != 23
        || sword.max_damage != 250
        || equipment.off_hand.is_some()
        || helmet.item_id != "minecraft:carved_pumpkin"
        || helmet.count != 1
        || helmet.damage != 0
        || helmet.max_damage != 0
        || equipment.chest.is_some()
        || leggings.item_id != "minecraft:iron_leggings"
        || leggings.damage != 31
        || leggings.max_damage != 225
        || boots.item_id != "minecraft:iron_boots"
        || boots.damage != 7
        || boots.max_damage != 195
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "the entity equipment snapshot did not match the six configured vanilla slots",
        ));
    }
    Ok("guest read six detached equipment slots with only canonical IDs and durability".to_owned())
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct EntityEquipmentWriteArguments {
    entity_id: String,
}

#[command(name = "entity.equipment.write")]
fn equips_only_an_empty_slot_and_preserves_the_existing_stack(
    arguments: EntityEquipmentWriteArguments,
) -> SjmlResult<String> {
    let handle = entity::loaded()?
        .into_iter()
        .find(|handle| {
            entity::snapshot(*handle).is_ok_and(|snapshot| {
                snapshot.uuid == arguments.entity_id
                    && snapshot.entity_type_id == "minecraft:zombie"
            })
        })
        .ok_or_else(|| {
            SjmlError::new(
                ErrorCode::InvalidArgument,
                "the equipment GameTest zombie is not loaded",
            )
        })?;

    if !entity::try_equip_default(handle, EntityEquipmentSlot::MainHand, "minecraft:iron_sword")? {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "the initially empty main-hand slot rejected its first default item",
        ));
    }
    let first_snapshot = entity::equipment(handle)?;
    let sword = first_snapshot.main_hand.as_ref().ok_or_else(|| {
        SjmlError::new(ErrorCode::InvalidArgument, "the main-hand item was not visible after insertion")
    })?;
    if sword.item_id != "minecraft:iron_sword" || sword.count != 1 || sword.damage != 0 {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity equipment readback did not match the default iron sword",
        ));
    }
    if entity::try_equip_default(handle, EntityEquipmentSlot::MainHand, "minecraft:diamond_sword")? {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "an occupied equipment slot accepted a replacement",
        ));
    }
    let final_snapshot = entity::equipment(handle)?;
    if final_snapshot.main_hand.as_ref().map(|stack| stack.item_id.as_str())
        != Some("minecraft:iron_sword")
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "refusing a replacement must leave the original equipment unchanged",
        ));
    }
    Ok("guest equipped one default item into an empty slot and rejected replacement".to_owned())
}
