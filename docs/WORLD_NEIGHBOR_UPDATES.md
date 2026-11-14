# Explicit vanilla neighbor updates

`world::notify_neighbors_at` asks Minecraft to run its ordinary neighbor-update logic around a
block position:

```rust
use sjml::{world, BlockPos, ErrorCode, SjmlError, SjmlResult};

fn refresh_neighbors(x: i32, y: i32, z: i32) -> SjmlResult<()> {
    let overworld = world::get("minecraft:overworld")?.ok_or_else(|| {
        SjmlError::new(ErrorCode::InvalidArgument, "the Overworld is not loaded")
    })?;
    world::notify_neighbors_at(overworld, BlockPos { x, y, z })
}
```

The call requires both `world.read` and `world.write`, a valid module-owned world handle, and the
owning logical-server thread. The target must be within build height and inside the current world
border. Its chunk and every in-height chunk containing one of its six direct neighbors must
already be loaded. The host checks these positions with non-loading chunk lookups before calling
vanilla `ServerLevel.updateNeighborsAt`; if any required chunk is unavailable, it returns
`InvalidArgument` without starting the update. X/Z coordinates are limited to
`-30_000_000..=30_000_000`.

Minecraft applies its normal neighbor callback rules, including redstone and trapdoor behavior.
This is a synchronous server-thread call, and the return value only indicates that Minecraft
accepted the update; vanilla callback failures map to the stable `UnknownErrorCode` ABI error.

The stable import is
`sjml_world_v1.notify_neighbors_at(handle_ptr, handle_len, x, y, z) -> i32`. It has no output
payload. Success is zero; failures use the standard ABI status code. The host does not load the
target or direct-neighbor chunks.

The dedicated-server GameTest places an iron trapdoor beside a redstone block without sending the
placement neighbor flag, then calls the API from the Rust guest and checks that vanilla opens the
trapdoor. It also asks the guest to update a distant unloaded position and verifies rejection leaves
that chunk unloaded. The test lives in
[`WorldNeighborUpdateGameTests`](../src/main/java/dev/somuch/sjml/neoforge/gametest/WorldNeighborUpdateGameTests.java).
