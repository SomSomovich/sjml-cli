use sjml::{
    inventory::{self, BlockPos},
    ErrorCode, SjmlError, SjmlResult, WorldHandle,
};

pub(super) fn verify_block_inventory_lore(
    world: WorldHandle,
    position: BlockPos,
    expected_custom_name: &str,
) -> SjmlResult<()> {
    let original = inventory::read_stack(world, position, None, 0)?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the chest fixture stack is missing",
        )
    })?;
    if original.item_id != "minecraft:diamond_pickaxe"
        || original.count != 1
        || original.damage != Some(37)
        || original.custom_name.as_deref() != Some(expected_custom_name)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "the original block-inventory item stack does not match the fixture",
        ));
    }

    let expected = vec!["Café 🪨".to_owned(), "§cstill literal".to_owned()];
    inventory::set_lore(world, position, 0, &expected)?;
    if inventory::read_lore(world, position, 0)? != expected {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "block-inventory lore did not round-trip as literal text",
        ));
    }
    if inventory::read_stack(world, position, None, 0)? != Some(original.clone()) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "the lore write changed item identity, count, damage, or name",
        ));
    }

    inventory::clear_lore(world, position, 0)?;
    if !inventory::read_lore(world, position, 0)?.is_empty() {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "clearing block-inventory lore left lines on the stack",
        ));
    }
    if inventory::read_stack(world, position, None, 0)? != Some(original) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "clearing block-inventory lore changed other item components",
        ));
    }
    Ok(())
}
