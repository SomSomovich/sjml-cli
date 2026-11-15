# Striking lightning

Server modules with the separate `world.lightning` capability can strike a
vanilla lightning bolt at a loaded block position:

```rust
use sjml::{world, BlockPos, SjmlResult, WorldHandle};

fn signal(world: WorldHandle, x: i32, y: i32, z: i32) -> SjmlResult<()> {
    world::strike_lightning(world, BlockPos { x, y, z }, true)
}
```

The module must request and receive `world.read` and `world.lightning`. The
block column's x/z must be within ±30,000,000, the y coordinate must be inside
build height, and the position must be inside the world border and an
already-loaded chunk. The call never requests a chunk load. The bolt spawns at
the bottom center of the requested block.

`visual_only = true` requests vanilla's visual-only bolt, which produces the
light and sound effects without setting fire or damaging entities.
`visual_only = false` leaves normal vanilla behavior intact, including entity
damage, lightning-rod interaction, and fire. Each module can strike at most four
bolts per logical-server tick; further attempts in the same tick return
`SERVICE_BUSY`.

The request uses ABI schema 1: version, module-owned world handle, three
little-endian `i32` block coordinates, and a Boolean visual-only flag. Every
value is validated before world access. Missing capabilities, out-of-border or
out-of-height positions, unloaded chunks, and malformed payloads return an
error.

GameTests spawn pigs, invoke real guest-created visual-only and damaging bolts,
and verify that only the damaging strike reduces health. Unit tests cover the
stable codec, bounds, capability enforcement, quota, static/client denial, and
wrong-thread rejection.
