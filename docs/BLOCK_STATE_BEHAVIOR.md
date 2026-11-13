# Exact-state block behavior

Registered blocks with declared Boolean or bounded integer state properties can assign a static
behavior override to one complete state with `registry::set_block_state_behavior`:

```rust
registry::add_block_state_property("machine", &BlockStateProperty::boolean("powered", false)?)?;
registry::set_block_state_behavior(
    "machine",
    &[("powered", "true")],
    BlockStateBehavior::new()
        .sound(BlockSound::Amethyst)
        .redstone_conductor(false)
        .suffocating(false)
        .view_blocking(false)
        .emissive_rendering(true)
        .post_process(true),
)?;
```

Each selector must provide every declared property exactly once, with canonical values in the
property's declared range. Multiple calls may add disjoint fields for the same exact state; a field
cannot be assigned twice. A state with no override, and any field left unset, retain the block's
existing behavior. Overrides are captured during `mod_register`, bounded to 512-byte ABI payloads,
and resolved through immutable host lookup tables. Minecraft queries never call guest code.

The state-aware vanilla hooks support the allowlisted sound groups, redstone conductor,
suffocation, view blocking, emissive rendering, and post-process predicates. Per-state friction,
walking-speed and jump factors, lava ignition, piston reaction, and destroy speed are unavailable
through NeoForge's 1.21.1 block API: the first three are block-wide `BlockBehaviour.Properties`
values, while ignition and piston reaction are cached on each `BlockState` from block-wide
properties. Destroy speed is also cached by `BlockState`; there is no state-aware block override.
SJML keeps these fields block-wide in `BlockBehavior` instead of adding a mixin or changing the
vanilla calculation path. Exact-state collision/selection shapes and light emission are exposed
separately by [`BLOCK_STATE_SHAPES.md`](BLOCK_STATE_SHAPES.md) and
[`BLOCK_STATE_LIGHT.md`](BLOCK_STATE_LIGHT.md).
