#![allow(dead_code)]

use sjml::inventory::{self, BlockPos};
use sjml::{SjmlResult, WorldHandle};

/// Read and update one registered enchantment on an existing container stack.
pub fn set_existing_stack_enchantment(
    world: WorldHandle,
    position: BlockPos,
    slot: u32,
    enchantment_id: &str,
    level: u32,
) -> SjmlResult<Option<u32>> {
    let previous = inventory::read_enchantment_level(world, position, slot, enchantment_id)?;
    inventory::set_enchantment_level(world, position, slot, enchantment_id, level)?;
    Ok(previous)
}

fn main() {}
