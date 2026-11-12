# Loaded block tag membership

Logical-server modules with `world.read` can check whether a block at a position
belongs to a vanilla, NeoForge, or data-pack block tag:

```rust
use sjml::{world, BlockPos, SjmlResult, WorldHandle};

fn is_log(world: WorldHandle, position: BlockPos) -> SjmlResult<Option<bool>> {
    world::block_has_tag(world, position, "minecraft:logs")
}
```

Pass the canonical `namespace:path` identifier without the `#` prefix. IDs are
limited to 256 UTF-8 bytes. The block is read from the current server tag set,
so datapack or resource reloads are reflected by later queries.

The function returns `Ok(Some(true))` for a member and `Ok(Some(false))` when a
loaded block is not a member. It returns `Ok(None)` if the position is outside
the world's build height or its chunk is not already loaded. The host never
loads a chunk. Invalid IDs, invalid world handles, permission failures, and
wrong-thread calls return an error.

The operation is server-only and requires a live module-owned world handle,
the server thread, and `world.read`. Unknown or currently empty tags simply do
not match. To enumerate bounded members, use the APIs described in
[`WORLD_TAG_MEMBERS.md`](WORLD_TAG_MEMBERS.md). Codec, import-boundary,
permission, unloaded-chunk, and dedicated server GameTests cover the contract.
