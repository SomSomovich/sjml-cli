use sjml::{inventory, BlockPos, ErrorCode, SjmlError, SjmlResult, WorldHandle};

pub(super) fn verify_block_inventory_enchantments(
    world: WorldHandle,
    position: BlockPos,
) -> SjmlResult<()> {
    let sharpness = "minecraft:sharpness";
    let unbreaking = "minecraft:unbreaking";
    if inventory::read_enchantment_level(world, position, 0, sharpness)?.is_some()
        || inventory::read_enchantment_level(world, position, 0, unbreaking)? != Some(2)
    {
        return Err(failure("the block-inventory sword must start with Unbreaking II only"));
    }
    inventory::set_enchantment_level(world, position, 0, sharpness, 3)?;
    if inventory::read_enchantment_level(world, position, 0, sharpness)? != Some(3)
        || inventory::read_enchantment_level(world, position, 0, unbreaking)? != Some(2)
    {
        return Err(failure("setting Sharpness must preserve Unbreaking II"));
    }
    inventory::remove_enchantment(world, position, 0, sharpness)?;
    if inventory::read_enchantment_level(world, position, 0, sharpness)?.is_some()
        || inventory::read_enchantment_level(world, position, 0, unbreaking)? != Some(2)
    {
        return Err(failure("removing Sharpness must preserve Unbreaking II"));
    }

    let mending = "minecraft:mending";
    if inventory::read_enchantment_level(world, position, 1, mending)?.is_some()
        || inventory::read_enchantment_level(world, position, 1, unbreaking)? != Some(1)
    {
        return Err(failure("the block-inventory enchanted book must start with Unbreaking I"));
    }
    inventory::set_enchantment_level(world, position, 1, mending, 1)?;
    if inventory::read_enchantment_level(world, position, 1, mending)? != Some(1)
        || inventory::read_enchantment_level(world, position, 1, unbreaking)? != Some(1)
    {
        return Err(failure("setting stored Mending must preserve Unbreaking I"));
    }
    inventory::remove_enchantment(world, position, 1, mending)?;
    if inventory::read_enchantment_level(world, position, 1, mending)?.is_some()
        || inventory::read_enchantment_level(world, position, 1, unbreaking)? != Some(1)
    {
        return Err(failure("removing stored Mending must preserve Unbreaking I"));
    }
    Ok(())
}

fn failure(message: &'static str) -> SjmlError {
    SjmlError::new(ErrorCode::InvalidArgument, message)
}
