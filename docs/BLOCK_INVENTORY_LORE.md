# Block inventory lore

The Rust SDK can read, replace, and clear the vanilla plain-text lore component on an item already
in a loaded block container. This API does not create stacks and does not expose arbitrary item
components.

```rust
use sjml::inventory::{self, BlockPos};
use sjml::{SjmlResult, WorldHandle};

fn label_existing_stack(world: WorldHandle, chest: BlockPos, slot: u32) -> SjmlResult<()> {
    let mut lines = inventory::read_lore(world, chest, slot)?;
    lines.push("Inspected by the server".to_owned());
    inventory::set_lore(world, chest, slot, &lines)
}

fn remove_label(world: WorldHandle, chest: BlockPos, slot: u32) -> SjmlResult<()> {
    inventory::clear_lore(world, chest, slot)
}
```

The imports are `sjml_inventory_v1.read_lore`, `set_lore`, and `clear_lore`. Reads require
`world.read` and `inventory.read`. Changes require `world.read` and `inventory.write`. These calls
run on the server thread and do not load chunks.

Lore reads and writes use the unsided Minecraft `Container` interface, like block item custom-name
updates. The block entity must already be loaded and implement `Container`. Reading an empty slot or
a stack without lore returns an empty vector. Setting lore and clearing lore require a non-empty
slot. Unresolved loot-table containers are rejected before their slots are inspected.

Each stack can have up to eight lore lines. A line can contain at most 256 UTF-8 bytes and 256
Unicode scalar values, and cannot contain control characters. Set one to eight lines; pass no lines
to `clear_lore`. Text is stored as literal components, so formatting codes remain ordinary text.
Read results contain flattened text with styles and click/hover events removed. Oversized lore
already present on a stack is rejected rather than truncated.

Only Minecraft's vanilla `LORE` component changes. The host copies the existing stack before the
update, preserving item identity, count, damage, name, enchantments, and all other components.
