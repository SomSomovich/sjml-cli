# Setting a block inventory stack count

Server modules can change the count of an existing stack in an already-loaded
block inventory with `inventory::set_count`:

```rust
use sjml::inventory::{self, BlockPos};
use sjml::{SjmlResult, WorldHandle};

fn keep_twelve_diamonds(world: WorldHandle) -> SjmlResult<()> {
    inventory::set_count(world, BlockPos { x: 12, y: 64, z: -5 }, 0, 12)
}
```

The module must request and receive `world.read` and `inventory.write`. The
count must be from 1 through 99 and fit both the registered item's current
maximum and the target slot's limit. The slot must already contain a registered
item. The operation preserves the item and every component; it cannot create a
new stack or clear a slot.

The block must be in a loaded chunk and expose a modifiable item-handler
capability. Unresolved random loot containers are rejected before their contents
are read. The host applies the replacement on the owning server thread and
checks the handler's result. If the handler rejects or throws, the host verifies
that the original stack remains or attempts to restore it; if restoration also
fails, the call returns an error instead of reporting success. Chunks are never
loaded by this operation.
