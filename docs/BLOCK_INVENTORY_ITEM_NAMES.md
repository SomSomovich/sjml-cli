# Block-inventory item names

Server modules can set or clear a plain display name on an existing item stack
in a block container:

```rust
use sjml::inventory::{self, BlockPos};
use sjml::{SjmlResult, WorldHandle};

fn label_and_restore(world: WorldHandle) -> SjmlResult<()> {
    let position = BlockPos { x: 12, y: 64, z: -5 };
    inventory::set_custom_name(world, position, 0, "Supplies")?;
    inventory::clear_custom_name(world, position, 0)
}
```

The module must request and receive both `world.read` and `inventory.write`.
Names are literal plain text, limited to 1,024 UTF-8 bytes and 256 Unicode
characters, and reject control characters. An empty name is invalid; use
`clear_custom_name` to remove the component.

The host requires an existing non-empty slot in a loaded block entity that
implements Minecraft's `Container` API. Writes are unsided and run on the
owning server thread. They copy the current stack, change only its vanilla
`CUSTOM_NAME` component, then replace the same slot and mark the container
changed. Item identity, count, durability, and unrelated components remain
intact. The host never creates an item or loads a chunk. Modded block entities
that expose only an `IItemHandler` and do not implement `Container` cannot be
changed through this API.

`inventory::read_stack` exposes the resulting name as bounded plain text. Styles,
click actions, hover contents, NBT, and unrelated vanilla components are not
made writable.
