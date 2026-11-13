# Registered block-state shapes

Blocks with declared Boolean or integer properties can use a different fixed
shape for any exact combination of those values. Declare every state property
first, then add one shape override per state that needs geometry different from
the fallback shape:

```rust
registry::register_block("machine", BlockDefinition::default())?;
registry::add_block_state_property(
    "machine",
    &BlockStateProperty::boolean("open", false)?,
)?;
registry::add_block_state_property(
    "machine",
    &BlockStateProperty::integer("stage", 0, 3, 0)?,
)?;
registry::set_block_state_shape(
    "machine",
    &[("open", "true"), ("stage", "3")],
    BlockShape::Empty,
)?;
```

`set_block_state_shape` is available only during `#[mod_register]` and requires
`content.register`. Each selector must include every property declared on the
block exactly once, with a canonical Boolean or in-range integer value. The
host rejects duplicate combinations and properties added after shape
overrides. A block can have at most 128 possible states; the host also caps
registered shape overrides at 16,384 across all packages. States without an
override use the ordinary block shape from `BlockDefinition` or
`set_block_shape`.

An override applies the same bounded `BlockShape` to selection, collision,
occlusion, and support queries. The supported forms are full cube, empty, and
one axis-aligned cuboid inside the unit block. Minecraft's ordinary block-state
updates and synchronization carry these values; the host evaluates a static
lookup and never calls guest code during rendering or collision checks.

The declarations are immutable after registry freeze. They do not provide
multi-box or rotated geometry, state-dependent movement/light/sound behavior,
custom placement rules, or arbitrary collision callbacks. See
[state-property declarations](BLOCK_STATE_PROPERTIES.md),
[fixed block shapes](WORLD_BLOCK_SHAPES.md), and
[block behavior](WORLD_BLOCK_BEHAVIOR.md). The GameTest in
[`BlockStatePropertiesGameTests`](../src/main/java/dev/somuch/sjml/neoforge/gametest/BlockStatePropertiesGameTests.java)
checks both fallback and overridden collision shapes through a live registered
block.
