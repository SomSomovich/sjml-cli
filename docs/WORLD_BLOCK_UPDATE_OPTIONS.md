# Block replacement update options

`world::set_block_with_options` replaces one loaded block with the default state of a registered block and lets the caller select three safe vanilla update bits.

```rust
use sjml::{world, BlockPos, BlockUpdateOptions};

world::set_block_with_options(
    overworld,
    BlockPos { x: 10, y: 64, z: 10 },
    "minecraft:redstone_block",
    BlockUpdateOptions {
        update_neighbors: true,
        update_clients: true,
        known_shape: false,
    },
)?;
```

The module must request and receive both `world.read` and `world.write`. The call is synchronous on the owning logical-server thread. It validates the world handle, block registry ID, build height, world border, and already-loaded target chunk; it never loads a chunk. Unloaded positions return `INVALID_ARGUMENT`.

`update_neighbors` sends the normal vanilla notifications to adjacent blocks. Disabling it can leave redstone, doors, fluids, and other dependent blocks unchanged until another update reaches them. `update_clients` sends the changed state to clients tracking the chunk; disabling it can leave their view stale until a later update packet. Keep both enabled for ordinary placement. `known_shape` maps to vanilla's `UPDATE_KNOWN_SHAPE` assertion and should be enabled only when the caller already established that the surrounding shape state is consistent.

The default [`world::set_block`](../sjml-sdk/src/world.rs) remains compatible and enables neighbor and client updates with `known_shape` disabled. The guest ABI accepts only the bit mask `0b1_0011` for neighbor (`1`), client (`2`), and known-shape (`16`) updates. Other bits are rejected, including flags for immediate updates, suppressing drops, invisible updates, and piston movement. `world::set_block_properties` continues to use the host's fixed `UPDATE_ALL` behavior.

See [the world API table](ABI.md#logical-server-worlds).
