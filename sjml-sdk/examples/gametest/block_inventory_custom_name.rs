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
            "operation must be either set or clear",
        )),
    }
}
