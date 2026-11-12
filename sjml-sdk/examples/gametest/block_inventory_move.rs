use sjml::{
    command,
    inventory::{self, BlockPos},
    world, ErrorCode, SjmlError, SjmlResult, StableDecode,
};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct BlockInventoryMoveArguments {
    source_x: i32,
    source_y: i32,
    source_z: i32,
    source_slot: u32,
    destination_x: i32,
    destination_y: i32,
    destination_z: i32,
    destination_slot: u32,
    count: u32,
}

#[command(name = "block_inventory.move_stack")]
fn move_block_inventory_stack(arguments: BlockInventoryMoveArguments) -> SjmlResult<String> {
    let world = world::get("minecraft:overworld")?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the real overworld must already be loaded",
        )
    })?;
    inventory::move_stack(
        world,
        BlockPos {
            x: arguments.source_x,
            y: arguments.source_y,
            z: arguments.source_z,
        },
        arguments.source_slot,
        BlockPos {
            x: arguments.destination_x,
            y: arguments.destination_y,
            z: arguments.destination_z,
        },
        arguments.destination_slot,
        arguments.count,
    )?;
    Ok("moved the requested block-inventory stack count".to_owned())
}
