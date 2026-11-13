#![allow(dead_code)]

use sjml::inventory::{self, BlockPos};
use sjml::{SjmlResult, WorldHandle};

/// Appends a line to an existing stack or clears that stack's vanilla lore.
pub fn update_existing_stack_lore(
    world: WorldHandle,
    position: BlockPos,
    slot: u32,
    clear: bool,
) -> SjmlResult<()> {
    if clear {
        return inventory::clear_lore(world, position, slot);
    }

    let mut lines = inventory::read_lore(world, position, slot)?;
    lines.push("Checked by the server".to_owned());
    inventory::set_lore(world, position, slot, &lines)
}

fn main() {}
