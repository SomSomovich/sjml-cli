# Registered block shapes

Modules with `content.register` can register a block with a fixed Minecraft
voxel shape during `#[mod_register]`:

```rust
use sjml::prelude::*;

#[mod_register]
fn register_content() -> SjmlResult<()> {
    registry::register_block_with_shape(
        "ruby_pedestal",
        BlockDefinition::default(),
        BlockShape::cuboid([0.25, 0.0, 0.25], [0.75, 0.5, 0.75])?,
    )
}
```

`BlockShape` accepts `FullCube`, `Empty`, or one `Cuboid`. Cuboid coordinates
are normalized to the block from 0.0 through 1.0 and must have finite,
strictly ordered corners. The same cuboid defines selection, collision,
occlusion, and support geometry. `register_block` keeps the full cube default;
`set_block_shape` can also be called after it during the same registration
callback.

For disconnected or more detailed geometry, use a bounded union of AABBs:

```rust
use sjml::prelude::*;

#[mod_register]
fn register_content() -> SjmlResult<()> {
    let shape = BlockShapeBoxes::new([
        BlockAabb::new([0.0, 0.0, 0.0], [0.2, 1.0, 1.0])?,
        BlockAabb::new([0.8, 0.0, 0.0], [1.0, 1.0, 1.0])?,
    ])?;
    registry::register_block_with_shape_boxes("ruby_grate", BlockDefinition::default(), &shape)
}
```

`BlockShapeBoxes` accepts 1 through 8 boxes. Every corner must be finite and
inside the normalized 0.0 through 1.0 block bounds, and each minimum must be
strictly below its corresponding maximum. Minecraft uses the union of the
boxes for selection, collision, occlusion, and support geometry. The existing
`set_block_shape` single-box API and ABI import remain available unchanged.

Shapes are immutable after the registry phase closes. Invalid paths, unknown
blocks, invalid coordinates, and calls outside `#[mod_register]` are rejected.
This API sets one fallback shape per block. For per-state geometry on blocks
with declared properties, see [registered block-state shapes](BLOCK_STATE_SHAPES.md).
See the Rust [`registry` API guide](../sjml-cli/docs/SDK_GUIDE.md#static-content-and-creative-tab-declarations)
and [ABI reference](ABI.md#static-registry-declarations).
