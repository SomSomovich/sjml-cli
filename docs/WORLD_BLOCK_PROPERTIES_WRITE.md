# Writing block-state properties

Use `world::set_block_properties` to change selected properties on the block already present at a
position. `world::block_properties` provides detached property names and values to inspect first.
The existing `world::set_block` behavior is unchanged: it replaces the target with a block's
default state.

```rust
use sjml::{inventory::BlockPos, world::{self, BlockProperty}};

world::set_block_properties(
    world_handle,
    BlockPos { x: 10, y: 64, z: 10 },
    "minecraft:oak_log",
    &[BlockProperty { name: "axis".into(), value: "x".into() }],
)?;
```

The module must request and receive both `world.read` and `world.write`. The call is synchronous
and runs on the owning server thread. It accepts 1–32 unique property updates, requires the exact
current block ID, and preserves every property omitted from the update list. The host validates
names and values against registered Minecraft properties, including enum choices and numeric
ranges. Unknown properties, duplicates, invalid values, or an expected-block mismatch return
`INVALID_ARGUMENT` without changing the block.

The operation rejects positions outside build height and chunks that are not already loaded; it
does not load chunks. The host always uses the fixed `Block.UPDATE_ALL` update flag. The stable-v1
request contains a versioned world handle and position, expected block ID, and a bounded vector of
versioned `{ name, value }` records. The maximum request size is 10,932 bytes. Minecraft objects
never cross the guest ABI.
