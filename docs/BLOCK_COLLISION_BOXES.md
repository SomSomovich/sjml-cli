# Block collision boxes

`world::block_collision_boxes(world, position)` reads the Minecraft collision shape for one
already-loaded block and returns detached `BlockCollisionBox` values in block-local coordinates.
The shape is queried with an empty collision context, so the result represents the block state's
ordinary collision geometry rather than geometry adjusted for a particular entity.

```rust
let Some(boxes) = world::block_collision_boxes(world, position)? else {
    // The target is outside build height, its chunk is unloaded, or the shape needed data from an
    // unloaded neighboring chunk.
    return Ok(());
};

for box_ in boxes {
    // Each min/max coordinate is relative to the block origin.
    let width = box_.max_x - box_.min_x;
    let height = box_.max_y - box_.min_y;
    let depth = box_.max_z - box_.min_z;
}
```

`Some(vec![])` means the block was available and has no collision shape, as with air. A full cube
is represented by one box from `(0, 0, 0)` to `(1, 1, 1)`. Slabs, fences, and registered SJML block
shapes can return partial or multiple boxes. The list is sorted by minimum coordinates followed by
maximum coordinates.

The API requires `world.read` and a logical-server context. It never loads a chunk. The host uses
Minecraft 1.21.1's `BlockState.getCollisionShape(..., CollisionContext.empty())` and visits the
shape boxes directly; if a shape query touches an unloaded neighbor, the result is unavailable.
At most 64 boxes are returned. Each coordinate must be finite and within ±16 blocks, and each box
must have positive extents no larger than 16 blocks. An unrepresentable shape fails with a bounded
ABI error instead of being truncated.
