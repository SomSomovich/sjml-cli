# Scheduled block ticks

Server modules can ask Minecraft to schedule one normal block tick at a position
that is already loaded:

```rust
use sjml::prelude::*;

fn queue_growth_tick(world: WorldHandle, position: BlockPos) -> SjmlResult<()> {
    world::schedule_block_tick(world, position, "minecraft:oak_sapling", 20)
}
```

The requested ID must match the block currently at the position. This check
prevents a delayed request from being applied to a different block after the
world changes. The host resolves the canonical block ID, verifies the position
is within build height, and checks the chunk through `getChunkNow`; it never
loads a chunk. Invalid IDs, unloaded positions, out-of-range coordinates, or a
block mismatch fail without scheduling a tick.

The module must request and receive both `world.read` and `world.write`. The
delay is 1 through 72,000 ticks, and one module can schedule at most 16 ticks
per logical-server tick. Work is passed to Minecraft's normal scheduled-block
tick queue, including its usual de-duplication and block tick behavior. This
does not call a guest callback or create a custom scheduler. The operation is
available only to active server modules on the owning server thread; static
registration receives a denied import.
