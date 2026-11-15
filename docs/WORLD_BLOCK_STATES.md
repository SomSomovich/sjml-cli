# Setting exact block states

Server modules with `world.read` and `world.write` can apply any parsed vanilla
block state to a loaded position:

```rust
use sjml::{world, BlockPos, SjmlResult, WorldHandle};

fn build_stairs(world: WorldHandle, x: i32, y: i32, z: i32) -> SjmlResult<()> {
    world::set_block_state(
        world,
        BlockPos { x, y, z },
        "minecraft:oak_stairs[facing=east,half=top,shape=straight,waterlogged=false]",
    )
}
```

The specification uses vanilla command syntax: a canonical `namespace:path` with
optional `[property=value,...]` entries. It is limited to 512 UTF-8 bytes and
never accepts NBT. The host parses it with Minecraft's own block-state parser,
so unknown blocks, unknown properties, or invalid values are rejected. The
position must be inside build height and the world border, and its chunk must
already be loaded; the call never requests a chunk load. The change uses normal
neighbor and client update flags.

A module can apply at most 32 block-state changes per logical-server tick;
further attempts return `SERVICE_BUSY`. The request is little-endian schema 1:
`u32` version, the 24-byte module-owned world handle, three `i32` block
coordinates, a `u32` UTF-8 byte length, and the state text. Missing
capabilities, static-registration calls, client contexts, unloaded positions,
and malformed specifications fail closed.

GameTests set an exact oak-stair state on a stone block and verify the block
type, facing, and half properties. Unit tests cover round-trips, truncation and
trailing bytes, NBT rejection, malformed property lists, and positions.
