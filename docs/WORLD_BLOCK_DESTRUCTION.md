# Destroying blocks

Server modules with `world.write` can destroy an existing block in a loaded
world:

```rust
use sjml::{world, BlockPos, SjmlResult, WorldHandle};

fn clear_wall(world: WorldHandle, x: i32, y: i32, z: i32) -> SjmlResult<()> {
    world::destroy_block(world, BlockPos { x, y, z }, true)
}
```

The module must request and receive both `world.read` and `world.write`. The
position must be inside build height and the world border, and its chunk must
already be loaded; the call never requests a chunk load. The position must
currently contain a non-air block. With `drop_items = true`, the normal vanilla
destroy path runs: block loot is spawned, block-entity contents are handled by
vanilla, neighbor updates apply, and the destroy game event is emitted. With
`drop_items = false`, the block is removed without spawning items.

This is a world mutation, not a player action: no player break event or tool
enchantment is involved, and drops use the empty-tool loot context. A module
can destroy at most 32 blocks per logical-server tick; further attempts return
`SERVICE_BUSY`. The request uses ABI schema 1: version, module-owned world
handle, three little-endian `i32` block coordinates, and a Boolean drop flag.
Missing capabilities, static-registration calls, client contexts, invalid or
out-of-border positions, unloaded chunks, and air targets fail closed.

GameTests place stone blocks, destroy one with drops and one without, and
verify the resulting air blocks and the vanilla cobblestone drop. The import
tests cover the stable codec, capability enforcement, quota, static/client
denial, malformed lengths, and wrong-thread rejection.
