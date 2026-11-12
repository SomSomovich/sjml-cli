# Bounded model animations

Guest entity and block-entity models can animate the whole model or individual
cuboids with deterministic client-side sine rotations. Each model part can have
one track per axis: X, Y, and Z. The runtime samples those tracks from the
Minecraft tick and partial tick; it does not call guest code during rendering.

```rust
use sjml::prelude::*;

#[mod_register]
fn register_model() -> SjmlResult<()> {
    registry::register_entity("clockwork_mite", EntityDefinition::default())?;

    let mut model = EntityModelDefinition::new();
    model.add_cuboid([-0.25, 0.0, -0.25], [0.25, 0.5, 0.25], 0xFFFF_8040)?;
    model.add_cuboid([-0.2, 0.5, -0.2], [0.2, 0.9, 0.2], 0xFF80_C0FF)?;

    // Whole-model motion, composed in X, then Y, then Z order.
    model.add_animation(ModelAnimation::sine_rotation(
        ModelAnimationAxis::Y, [0.0, 0.25, 0.0], 30.0, 80, 0,
    )?)?;
    model.add_animation(ModelAnimation::sine_rotation(
        ModelAnimationAxis::Z, [0.0, 0.25, 0.0], 8.0, 40, 10,
    )?)?;

    // Cuboid index 1 rotates around its own local pivot while the first stays still.
    model.add_cuboid_animation(1, ModelAnimation::sine_rotation(
        ModelAnimationAxis::X, [0.0, 0.5, 0.0], 20.0, 32, 8,
    )?)?;
    registry::register_entity_model("clockwork_mite", model)?;
    Ok(())
}
```

`add_animation` and `add_cuboid_animation(index, …)` replace a track on the
same axis and preserve the other axes. `clear_animation(axis)` and
`clear_cuboid_animation_axis(index, axis)` remove one axis. The legacy
`set_animation` and `set_cuboid_animation` calls still install one track, and
clear any other tracks on that model part. Cuboid indices are zero-based and
follow the order of `add_cuboid` calls.

Each track is bounded: pivot coordinates must be finite and stay in the model's
local bounds (entity models use `[-4, 4]`; block-entity models use `[0, 1]`),
amplitude is from -180 to 180 degrees, period is 1 to 12,000 game ticks, and
phase is less than the period. A model has at most 32 cuboids and at most three
tracks per model part. The renderer evaluates at most 99 tracks per model
(three for the parent and three for each cuboid), with no per-frame guest call.

Static model payloads keep schema 1. Existing single model rotations and
per-cuboid rotations keep schemas 2 and 3, respectively. Schema 4 stores fixed
X/Y/Z slots for the model and each cuboid. It is bounded to roughly 4 KiB for a
32-cuboid model and is decoded by the host before registration is staged.

The schema-4 payload starts with a little-endian `u32` schema (`4`) and a
little-endian `u32` cuboid count. Each cuboid then has six little-endian `f32`
coordinates (`min XYZ`, `max XYZ`) and one little-endian `u32` ARGB color. The
remaining bytes contain three 32-byte animation slots for the model, followed
by three slots for each cuboid in declaration order. Slots always appear as X,
Y, then Z. A slot begins with a little-endian `u32` marker. Marker `0` means
the rest of the slot is zero padding; marker `1` is followed by `u32 axis`,
three `f32` pivot coordinates, `f32` amplitude in degrees, `u32` period ticks,
and `u32` phase ticks. The encoded axis must match its slot. Any other marker,
nonzero padding, or mismatched axis is rejected before staging the model.
