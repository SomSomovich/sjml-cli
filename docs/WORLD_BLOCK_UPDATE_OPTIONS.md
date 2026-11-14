# Block replacement update options

`world::set_block_with_options` replaces one loaded block with the default state of a registered block and lets the caller select four safe vanilla update options.

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
        suppress_drops: false,
    },
)?;
```

The module must request and receive both `world.read` and `world.write`. The call is synchronous on the owning logical-server thread. It validates the world handle, block registry ID, build height, world border, and already-loaded target chunk; it never loads a chunk. Unloaded positions return `INVALID_ARGUMENT`.

`update_neighbors` sends the normal vanilla notifications to adjacent blocks. Disabling it can leave redstone, doors, fluids, and other dependent blocks unchanged until another update reaches them. `update_clients` sends the changed state to clients tracking the chunk; disabling it can leave their view stale until a later update packet. Keep both enabled for ordinary placement. `known_shape` maps to vanilla's `UPDATE_KNOWN_SHAPE` assertion and should be enabled only when the caller already established that the surrounding shape state is consistent. `suppress_drops` maps to `UPDATE_SUPPRESS_DROPS` and scopes drop suppression to synchronous neighbor reactions from this call. Nested `Level.destroyBlock(..., drop = true, ...)` calls do not spawn drops while the update is active; later or unrelated destruction is unaffected.

The default [`world::set_block`](../sjml-cli/sjml-sdk/src/world.rs) remains compatible and enables neighbor and client updates with the other options disabled. The guest ABI accepts only the bit mask `0b10_0011`: neighbor (`1`), client (`2`), known-shape (`16`), and suppress-drops (`32`). Flags for invisible updates, immediate updates, and piston movement remain unavailable. `world::set_block_properties` continues to use the host's fixed `UPDATE_ALL` behavior.

See [the world API table](ABI.md#logical-server-worlds) and [`WorldBlockUpdateOptionsGameTests`](../src/main/java/dev/somuch/sjml/neoforge/gametest/WorldBlockUpdateOptionsGameTests.java).
