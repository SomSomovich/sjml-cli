# Bounded model animations

Guest entity and block-entity models can animate the whole model or individual
cuboids with deterministic client-side sine rotations and periodic keyframe
rotation, translation, and scale tracks. The runtime samples those tracks from
the Minecraft tick and partial tick; it does not call guest code during rendering.

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
phase is less than the period. A model has at most 32 cuboids. Each model part
can have up to three rotation tracks, one vector translation track, and one
vector scale track. A keyframe model is capped at 165 tracks total; the renderer
can evaluate at most 264 tracks including sine rotations, keyframe rotations,
translations, and scales. Module and host keyframe budgets are 512/8,192 and
8,192/65,536 tracks/frames, respectively. Rendering makes no per-frame guest
call.

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


## Periodic keyframe rotations

Models can also receive keyframe tracks after `register_entity_model` or
`register_block_entity_model` in the same `#[mod_register]` callback. The host samples the periodic
curve using the current game tick and partial tick, so no guest callback runs in the render loop.

```rust
use sjml::prelude::*;

#[mod_register]
fn register_animated_model() -> SjmlResult<()> {
    registry::register_entity("clockwork_mite", EntityDefinition::default())?;
    let mut model = EntityModelDefinition::new();
    model.add_cuboid([-0.25, 0.0, -0.25], [0.25, 0.5, 0.25], 0xFFFF_8040)?;
    registry::register_entity_model("clockwork_mite", model)?;

    let wing = ModelKeyframeTrack::new(
        ModelAnimationAxis::Z,
        [0.0, 0.25, 0.0],
        12,
        vec![ModelKeyframe::new(0, -35.0)?, ModelKeyframe::new(6, 35.0)?],
    )?;
    model_animation::register_entity_model_keyframe_track(
        "clockwork_mite",
        ModelAnimationTarget::Cuboid(0),
        wing,
    )
}
```

`ModelAnimationTarget::Model` targets the whole model; `Cuboid(index)` targets one zero-based
cuboid. Every target can use at most one track per axis across sine and keyframe rotations. A track
has 2–32 frames, starts at tick zero, uses strictly increasing frame ticks below a 2–12,000 tick
period, and angles from -180 to 180 degrees. Adjacent frames are linearly interpolated; the final
segment wraps from the last frame to the first. Entity pivots stay within `[-4, 4]`; block-entity
pivots stay within `[0, 1]`. The display-backed client must verify visible model movement.

## Translation and scale keyframes

`ModelTransformKeyframeTrack::translation` animates an XYZ offset and
`ModelTransformKeyframeTrack::scale` animates positive XYZ scale factors around
a pivot. Each target (the whole model or one cuboid) can have one track of each
kind. All three components share the same keyframe times, so one track
describes a synchronized XYZ path. Values are linearly interpolated and wrap at
the declared duration. The renderer applies translation, scale, then the
existing rotation tracks; scale keeps its pivot fixed.

Translation is in model-local units and is limited to `[-4, 4]` per component
for entity models and `[-1, 1]` for block-entity models. Scale factors are
limited to `[0.01, 16]`; zero, negative, non-finite, and larger factors are
rejected. Scale pivots use the existing entity `[-4, 4]` or block-entity
`[0, 1]` bounds. Each track contains 2–32 frames, starts at tick zero, and has
a 2–12,000 tick duration. The new `register_model_transform_keyframe_track`
import carries its own schema-1 payload; the existing rotation import and its
rotation-only payload remain unchanged.

The transform payload has a 36-byte little-endian header: schema (`1`), target
kind (`0` entity, `1` block entity), cuboid index (`0xFFFFFFFF` for the whole
model), transform kind (`0` translation, `1` scale), pivot XYZ, duration ticks,
and frame count. Each following 16-byte frame stores a tick and three `f32`
values (X, Y, Z). Translation requires a zero pivot. The host checks the exact
payload length, target ownership, model/cuboid existence, value ranges, and
per-model/module/global budgets before staging a track.

```rust
use sjml::prelude::*;

let translation = ModelTransformKeyframeTrack::translation(
    40,
    vec![
        ModelTransformKeyframe::new(0, [0.0, 0.0, 0.0])?,
        ModelTransformKeyframe::new(20, [0.0, 0.25, 0.0])?,
    ],
)?;
model_animation::register_entity_model_transform_keyframe_track(
    "clockwork_mite", ModelAnimationTarget::Cuboid(0), translation,
)?;

let scale = ModelTransformKeyframeTrack::scale(
    [0.0, 0.25, 0.0],
    40,
    vec![
        ModelTransformKeyframe::new(0, [1.0, 1.0, 1.0])?,
        ModelTransformKeyframe::new(20, [1.2, 1.0, 1.2])?,
    ],
)?;
model_animation::register_entity_model_transform_keyframe_track(
    "clockwork_mite", ModelAnimationTarget::Cuboid(0), scale,
)?;
```
