use sjml::{
    command,
    inventory::{self, BlockPos},
    world, ErrorCode, SjmlError, SjmlResult, StableDecode,
};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct BlockInventoryCustomNameArguments {
    operation: String,
    chest_x: i32,
    chest_y: i32,
    chest_z: i32,
}

#[command(name = "block_inventory.custom_name")]
fn update_block_inventory_custom_name(
    arguments: BlockInventoryCustomNameArguments,
) -> SjmlResult<String> {
    let world = world::get("minecraft:overworld")?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the real overworld must already be loaded",
        )
    })?;
    let position = BlockPos {
        x: arguments.chest_x,
        y: arguments.chest_y,
        z: arguments.chest_z,
    };
    match arguments.operation.as_str() {
        "count" => {
            let original = inventory::read_stack(world, position, None, 2)?.ok_or_else(|| {
                SjmlError::new(ErrorCode::InvalidArgument, "count test stack was missing")
            })?;
            if original.count != 6
                || original.custom_name.as_deref() != Some("count-preserved block name")
                || inventory::read_lore(world, position, 2)? != ["count-preserved block lore"]
                || inventory::read_float_component(world, position, None, 2, "gametest_heat")?
                    .is_some()
            {
                return Err(SjmlError::new(
                    ErrorCode::InvalidArgument,
                    "block inventory count fixture did not match its original components",
                ));
            }
            inventory::write_float_component(world, position, 2, "gametest_heat", 1.5)?;
            if inventory::read_float_component(world, position, None, 2, "gametest_heat")?
                != Some(1.5)
            {
                return Err(SjmlError::new(
                    ErrorCode::InvalidArgument,
                    "block inventory float component did not round-trip through the guest API",
                ));
            }
            inventory::set_count(world, position, 2, 12)?;
            let updated = inventory::read_stack(world, position, None, 2)?.ok_or_else(|| {
                SjmlError::new(
                    ErrorCode::InvalidArgument,
                    "updated count test stack was missing",
                )
            })?;
            if updated.item_id != "minecraft:stone"
                || updated.count != 12
                || updated.custom_name.as_deref() != Some("count-preserved block name")
                || inventory::read_lore(world, position, 2)? != ["count-preserved block lore"]
                || inventory::read_float_component(world, position, None, 2, "gametest_heat")?
                    != Some(1.5)
            {
                return Err(SjmlError::new(
                    ErrorCode::InvalidArgument,
                    "block inventory count update changed item identity or components",
                ));
            }
            let too_many = inventory::set_count(world, position, 2, 65).unwrap_err();
            if too_many.code() != ErrorCode::InvalidArgument {
                return Err(SjmlError::new(
                    ErrorCode::InvalidArgument,
                    "block inventory count must obey item and slot limits",
                ));
            }
            Ok("block inventory float component and count update preserved item data".to_owned())
        }
        "damage" => {
            let original = inventory::read_stack(world, position, None, 0)?.ok_or_else(|| {
                SjmlError::new(
                    ErrorCode::InvalidArgument,
                    "durability test sword was missing",
                )
            })?;
            if original.item_id != "minecraft:diamond_sword"
                || original.count != 1
                || original.damage != Some(17)
                || original.max_damage.is_none()
                || original.custom_name.as_deref() != Some("preserve block sword name")
                || inventory::read_lore(world, position, 0)? != ["durability-preserved lore"]
                || inventory::read_enchantment_level(world, position, 0, "minecraft:unbreaking")?
                    != Some(2)
            {
                return Err(SjmlError::new(
                    ErrorCode::InvalidArgument,
                    "block inventory durability fixture did not match its original data",
                ));
            }

            inventory::set_damage(world, position, 0, 30)?;
            let updated = inventory::read_stack(world, position, None, 0)?.ok_or_else(|| {
                SjmlError::new(
                    ErrorCode::InvalidArgument,
                    "updated durability sword was missing",
                )
            })?;
            if updated.item_id != original.item_id
                || updated.count != original.count
                || updated.damage != Some(30)
                || updated.max_damage != original.max_damage
                || updated.custom_name != original.custom_name
                || inventory::read_lore(world, position, 0)? != ["durability-preserved lore"]
                || inventory::read_enchantment_level(world, position, 0, "minecraft:unbreaking")?
                    != Some(2)
            {
                return Err(SjmlError::new(
                    ErrorCode::InvalidArgument,
                    "block inventory durability update changed unrelated stack data",
                ));
            }

            let maximum = updated.max_damage.ok_or_else(|| {
                SjmlError::new(
                    ErrorCode::InvalidArgument,
                    "host omitted the maximum durability for a damageable sword",
                )
            })?;
            let invalid = inventory::set_damage(world, position, 0, maximum).unwrap_err();
            if invalid.code() != ErrorCode::InvalidArgument {
                return Err(SjmlError::new(
                    ErrorCode::InvalidArgument,
                    "block inventory durability must reject damage at maximum durability",
                ));
            }
            Ok(
                "block inventory durability changed only damage and rejected a broken stack"
                    .to_owned(),
            )
        }
        "enchantment" => {
            super::block_inventory_enchantments::verify_block_inventory_enchantments(
                world, position,
            )?;
            Ok(
                "block inventory enchantment read, set, preservation, and removal succeeded"
                    .to_owned(),
            )
        }
        "set" => {
            inventory::set_custom_name(world, position, 0, "Café 🪨")?;
            super::block_inventory_lore::verify_block_inventory_lore(world, position, "Café 🪨")?;
            Ok("set the block-inventory custom name and verified lore operations".to_owned())
        }
        "clear" => {
            inventory::clear_custom_name(world, position, 0)?;
            Ok("cleared the block-inventory custom name".to_owned())
        }
        _ => Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "operation must be set, clear, enchantment, or count",
        )),
    }
}
