# Overworld spawn point

The SDK reads and updates the configured vanilla spawn position and yaw for a loaded Overworld:

```rust
use sjml::{world, SjmlError, ErrorCode, WorldSpawnPoint};

let overworld = world::get("minecraft:overworld")?.ok_or_else(|| {
    SjmlError::new(ErrorCode::InvalidArgument, "the Overworld is not loaded")
})?;
let current = world::spawn_point(overworld)?;
let replacement = WorldSpawnPoint::new(0, 80, 0, 45.0)?;
world::set_spawn_point(overworld, replacement)?;
```

Reads require `world.read`; writes require `world.write` and a read-scoped handle so the host can
validate ownership and that the handle refers specifically to `minecraft:overworld`. Both calls
run on the owning server thread. Other dimensions, stale handles, and client contexts are rejected.

The typed snapshot contains integer block coordinates and a finite yaw in degrees. The host checks
X/Z against Minecraft's world coordinate limit and current world border, and checks Y against the
loaded Overworld's dynamic build-height range. It performs no block or heightmap lookup. Reads use
the stored level data directly; they do not call `getSharedSpawnPos()`, whose out-of-border fallback
can query terrain. Writes use `ServerLevel.setDefaultSpawnPos`, so Minecraft broadcasts the vanilla
spawn update and adjusts its spawn-region ticket. No guest-directed chunk query or synchronous chunk
load is performed; vanilla may load spawn chunks as part of its normal ticket handling.
