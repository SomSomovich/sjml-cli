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

Shapes are immutable after the registry phase closes. Invalid paths, unknown
blocks, invalid coordinates, and calls outside `#[mod_register]` are rejected.
This API sets one fallback shape per block and does not expose multiple
disconnected boxes. For per-state geometry on blocks with declared properties,
see [registered block-state shapes](BLOCK_STATE_SHAPES.md).
See the Rust [`registry` API guide](../sjml-cli/docs/SDK_GUIDE.md#static-content-and-creative-tab-declarations)
and [ABI reference](ABI.md#static-registry-declarations).
