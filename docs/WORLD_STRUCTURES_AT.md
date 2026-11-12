# Structure lookup at a loaded position

Server modules with `world.read` can ask which generated structure pieces contain one block
position:

```rust
use sjml::{host::{self, LogLevel}, world, BlockPos, SjmlResult, WorldHandle};

fn inspect(world_handle: WorldHandle) -> SjmlResult<()> {
    match world::structures_at(world_handle, BlockPos { x: 0, y: 70, z: 0 })? {
        Some(ids) if ids.is_empty() => host::log(LogLevel::Info, "position is outside structures")?,
        Some(ids) => host::log(LogLevel::Info, &format!("position belongs to {ids:?}"))?,
        None => host::log(LogLevel::Info, "position or a referenced structure start is not loaded")?,
    }
    Ok(())
}
```

The host checks the target chunk and each referenced structure-start chunk with the server chunk
cache's non-loading `getChunkNow` path. If any required chunk is unavailable, it returns `None`
instead of asking Minecraft to load it or returning a partial answer. A loaded position with no
matching structure returns `Some(vec![])`. Querying outside the dimension build height also
returns `None`.

The guest receives only canonical structure registry IDs, sorted lexically and capped at 128.
The host caps work at 4,096 chunk references per query and rejects a larger result. Coordinates
outside Minecraft's ±30,000,000 horizontal world bound are rejected. This reads already-generated
structure pieces; it does not predict where an ungenerated structure could start and does not
query structure-placement search logic.

The operation requires a logical-server context, the owning server thread, a live module-owned
world handle, and `world.read`. It is unavailable during static registration and on physical
clients. Tests cover the versioned codec, bounds, permissions, ownership, thread checks, and guest
host-import path; display-backed client testing does not apply to this server-only query.
