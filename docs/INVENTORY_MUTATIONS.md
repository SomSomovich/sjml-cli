# Block inventory transfers

SJML exposes one bounded block-inventory mutation: `sjml::inventory::transfer`. It moves items
from one block-handler slot to another without exposing a Minecraft `ItemStack` or returning a
copyable item value to the module.

```rust
use sjml::inventory::{self, BlockPos, Direction};

let source = inventory::InventoryEndpoint {
    position: BlockPos { x: 10, y: 64, z: 10 },
    side: Some(Direction::East),
    slot: 0,
};
let destination = inventory::InventoryEndpoint {
    position: BlockPos { x: 11, y: 64, z: 10 },
    side: Some(Direction::West),
    slot: 2,
};
let moved = inventory::transfer(world, source, destination, 16)?;
```

The call returns the number actually moved. It returns zero if either loaded position has no
matching item-handler capability, the source slot is empty, or the destination cannot accept
items. Invalid slots, equal source and destination slots on the same handler, and counts outside
`1..=64` return an error. The host validates that the source item has a canonical registered
item ID, but modules do not supply an item ID that could mint new stacks.

The function requires both `world.read` and `inventory.write`. It is synchronous and runs through
the module's owning server-world handle, so it inherits that handle's server-thread and lifetime
checks. It queries only chunks already loaded at both positions and never requests a chunk load.
`None` for a side queries the unsided capability; a direction queries the handler exposed on that
face.

The host simulates extraction and insertion before changing either slot, then performs one transfer
on the source and destination capability. It passes stack copies to item handlers and never stores
or transfers a guest-owned stack reference. If an actual insertion unexpectedly returns a partial
remainder after simulation, SJML attempts to restore that remainder to the source slot and reports
the count accepted by the destination. This relies on the NeoForge `IItemHandler` contract: a
handler must honor simulation and must return the remainder of its insertion. The ABI does not
support arbitrary multi-slot transactions or inventory writes from an item ID and count.

## ABI shape

The import is `sjml_inventory_v1.transfer`, returning an `i32` status. Its 15 `i32` parameters are,
in order: world pointer/length; source position pointer/length; source side pointer/length; source
slot; destination position pointer/length; destination side pointer/length; destination slot;
requested count; output pointer/capacity. The output is one little-endian `u32` moved count. Each
position and side uses the existing stable encoding documented by the SDK types. The output span is
validated before any item-handler mutation.

Modules declare both permissions in the module manifest:

```toml
[[modules]]
id = "main"
path = "modules/main.wasm"
side = "server"
permissions = ["world.read", "inventory.write"]
```
