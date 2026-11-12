use sjml::{
    command,
    inventory::{self, BlockPos},
    world, ErrorCode, SjmlError, SjmlResult, StableDecode,
};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct BlockInventoryCustomNameArguments {
    chest_x: i32,
    chest_y: i32,
    chest_z: i32,
}

#[command(name = "block_inventory.custom_name.set")]
fn set_block_inventory_custom_name(
    arguments: BlockInventoryCustomNameArguments,
) -> SjmlResult<String> {
    let world = world::get("minecraft:overworld")?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the real overworld must already be loaded",
        )
    })?;
    inventory::set_custom_name(
        world,
        BlockPos {
            x: arguments.chest_x,
            y: arguments.chest_y,
            z: arguments.chest_z,
        },
        0,
        "Café 🪨",
    )?;
    Ok("set the block-inventory custom name".to_owned())
}

#[command(name = "block_inventory.custom_name.clear")]
fn clear_block_inventory_custom_name(
    arguments: BlockInventoryCustomNameArguments,
) -> SjmlResult<String> {
    let world = world::get("minecraft:overworld")?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the real overworld must already be loaded",
        )
    })?;
    inventory::clear_custom_name(
        world,
        BlockPos {
            x: arguments.chest_x,
            y: arguments.chest_y,
            z: arguments.chest_z,
        },
        0,
    )?;
    Ok("cleared the block-inventory custom name".to_owned())
}
