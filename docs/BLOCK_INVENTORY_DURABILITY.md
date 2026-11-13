# Changing block inventory durability

Server modules can set durability on an existing tool in an already-loaded
block container with `inventory::set_damage`:

```rust
use sjml::inventory::{self, BlockPos};
use sjml::{SjmlResult, WorldHandle};

fn repair_tool(world: WorldHandle) -> SjmlResult<()> {
    inventory::set_damage(world, BlockPos { x: 12, y: 64, z: -5 }, 0, 0)
}
```

The module must request and receive `world.read` and `inventory.write`. The
slot must contain a nonempty damageable item. The damage value must be from 0
through 65,535 and strictly below that item's maximum durability. Empty slots,
nondamageable items, and values at or above maximum durability are errors.

The host copies the existing stack and changes only its damage component,
preserving the item, count, and other components. The block must be in a loaded
chunk, implement `Container`, and have resolved random loot. The operation runs
synchronously on the owning server thread and never loads chunks.
