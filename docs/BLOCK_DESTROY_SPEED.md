# Loaded block destroy speed

The Rust SDK exposes `world::block_destroy_speed(world, position)` to read the
vanilla destroy-speed value of a block state at one already-loaded position:

```rust
let speed = sjml_sdk::world::block_destroy_speed(world, position)?;
match speed {
    None => { /* outside build height or the chunk is not loaded */ }
    Some(-1.0) => { /* vanilla treats the block as unbreakable */ }
    Some(value) => { /* finite block hardness/destroy speed in 0..=1_000_000 */ }
}
```

The query requires `world.read` and logical-server context. It checks the
position's build height and loaded-chunk state before reading the block; it
never loads or generates a chunk. Values are taken from Minecraft's
`BlockState.getDestroySpeed` and do not account for a player's tool, effects,
or enchantments. Negative vanilla values are normalized to `-1.0`; non-finite
or greater than one million values from another mod are rejected.

This is a read-only query. It does not change block hardness or control whether
a player can break a block.
