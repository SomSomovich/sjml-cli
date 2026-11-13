# Writing block-state properties

`world::set_block_properties` changes selected properties on the block already present at a
position. It complements `world::block_properties`, which returns detached property name/value
pairs, and the existing `world::set_block`, which still replaces a block with its default state.

```rust
use sjml::{inventory::BlockPos, world::{self, BlockProperty}};

world::set_block_properties(
    world_handle,
    BlockPos { x: 10, y: 64, z: 10 },
    "minecraft:oak_log",
    &[BlockProperty { name: "axis".into(), value: "x".into() }],
)?;
```

The module must declare and receive both `world.read` and `world.write`. The call is synchronous,
server-thread-only, and accepts 1–32 unique updates. Pass the exact current block ID as
`expected_block_id`; if another action changed that block before the call, the host rejects the
request. The host validates every property name and value against that block's registered
Minecraft properties, including enum members and integer ranges. Omitted properties retain their
current values. A block is never replaced by this API.

The host rejects unknown properties, duplicate names, invalid values, mismatched block IDs,
out-of-height positions, and unloaded chunks. It checks chunk residency without loading a chunk.
Minecraft receives a fixed `Block.UPDATE_ALL` update flag for this property-only operation. Guests
cannot choose its neighbor or client notification behavior. The separate
[`world::set_block_with_options`](WORLD_BLOCK_UPDATE_OPTIONS.md) API controls a block replacement's
safe neighbor/client/known-shape flags. `INVALID_ARGUMENT` indicates rejected input or a stale
expected block and leaves the block unchanged.

The stable-v1 request is a versioned record containing the world handle, versioned block position,
expected block ID, and a vector of versioned `{ name, value }` properties. The maximum encoded
request is 10,932 bytes. The ABI deliberately transfers strings and never exposes Minecraft state
objects.
