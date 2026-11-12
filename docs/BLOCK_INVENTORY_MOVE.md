# Moving items between block inventory slots

Server modules can move an exact count between two slots in already-loaded
block containers with `inventory::move_stack`:

```rust
use sjml::inventory::{self, BlockPos};
use sjml::{SjmlResult, WorldHandle};

fn move_supplies(world: WorldHandle) -> SjmlResult<()> {
    inventory::move_stack(
        world,
        BlockPos { x: 12, y: 64, z: -5 },
        0,
        BlockPos { x: 13, y: 64, z: -5 },
        2,
        4,
    )
}
```

The module must request and receive `world.read` and `inventory.write`. The
requested count must be 1 through 64, and the source slot must already contain
at least that many items. Both endpoints must be distinct valid slots in loaded
block entities implementing Minecraft's `Container` API. The destination must
be empty or contain the same registered item with exactly matching components.
The resulting count must fit the destination container and item's stack limit.

Both endpoint states and limits are checked before either slot changes. The
operation moves the exact requested count or returns an error; it does not
partially move items. It copies the source stack's components to an empty
destination, preserves matching destination components, and only changes stack
counts. It does not expose a detached stack to the module or create an item.
Unresolved random loot containers are rejected before their contents can be
generated. It never loads chunks, runs synchronously on the owning server thread, and calls
`setChanged` on each affected container after the slot updates (a shared
container is explicitly marked once).
