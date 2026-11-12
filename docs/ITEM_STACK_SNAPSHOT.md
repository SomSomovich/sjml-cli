# Block item-stack snapshots

Server-side modules can inspect one slot in an ordinary loaded block item
handler through `inventory::read_stack`:

```rust
use sjml::inventory::{self, BlockPos};
use sjml::{SjmlResult, WorldHandle};

fn inspect_first_slot(world: WorldHandle) -> SjmlResult<()> {
    let position = BlockPos { x: 12, y: 64, z: -5 };
    if let Some(stack) = inventory::read_stack(world, position, None, 0)? {
        sjml::host::log(sjml::LogLevel::Info, &format!(
            "{} x{} (max {})", stack.item_id, stack.count, stack.max_stack_size
        ))?;
    }
    Ok(())
}
```

The module must request and receive both `world.read` and `inventory.read`.
The host resolves the module-owned live world handle on the server thread,
checks only the already-loaded chunk, and queries the usual sided or unsided
item-handler capability. It never loads a chunk. An unloaded position, absent
handler, or empty slot returns `Ok(None)`; an out-of-range slot is an error.
Slots are limited to indices 0 through 4095.

`ItemStackSnapshot` contains a canonical item ID (at most 256 UTF-8 bytes),
count, stack maximum, slot limit, optional current and maximum durability, and
an optional custom display name. Existing stacks in `Container` block entities
can set or clear that name with [`inventory::set_custom_name` and
`inventory::clear_custom_name`](BLOCK_INVENTORY_ITEM_NAMES.md). Durability values
are present only for damageable stacks. The name is flattened to plain text
without style, click, or hover data, normalized to Unicode NFC, stripped of
control/format/line-break
characters, and limited to 1024 UTF-8 bytes. Non-damageable stacks and stacks
without a custom name use `None` for those fields.

The stable schema v1 order is `item_id`, `count`, `max_stack_size`,
`slot_limit`, `damage`, `max_damage`, and `custom_name`; the outer optional
value distinguishes an absent or empty slot from a present stack. This API
does not expose NBT or arbitrary data components. Rust validates the canonical
item ID, optional durability pair, name size, and 4096-slot bound after decode.
