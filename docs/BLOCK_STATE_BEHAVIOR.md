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
        .post_process(true)
        .push_reaction(BlockPushReaction::Destroy),
)?;
```

Each selector must provide every declared property exactly once, with canonical values in the
property's declared range. Multiple calls may add disjoint fields for the same exact state; a field
cannot be assigned twice. A state with no override, and any field left unset, retain the block's
existing behavior. Overrides are captured during `mod_register`, bounded to 512-byte ABI payloads,
and resolved through immutable host lookup tables. Minecraft queries never call guest code.

The state-aware vanilla hooks support the allowlisted sound groups, redstone conductor,
suffocation, view blocking, emissive rendering, post-process predicates, friction, flammability,
fire-spread speed, and piston reaction. For piston behavior, NeoForge 1.21.1 calls `IBlockExtension#getPistonPushReaction`
with the current `BlockState`; SJML returns the configured enum for an exact state and `null`
for states without an override, preserving the block-wide `BlockBehavior` fallback. This is a
static lookup and never calls guest code from piston checks. Flammability and fire-spread speed
use NeoForge's state-aware `IBlockExtension#getFlammability` and
`IBlockExtension#getFireSpreadSpeed` hooks; unset fields defer to the block's existing values.
Per-state walking-speed and jump factors, lava ignition, and destroy speed remain unavailable:
the first two are block-wide `BlockBehaviour.Properties` values, while lava ignition and destroy
speed are cached by each `BlockState` without a corresponding state-aware hook. SJML does not add a mixin or change the
vanilla calculation path. Exact-state collision/selection shapes and light emission are exposed
separately by [`BLOCK_STATE_SHAPES.md`](BLOCK_STATE_SHAPES.md),
[`BLOCK_STATE_LIGHT.md`](BLOCK_STATE_LIGHT.md), and
[`BLOCK_STATE_FIRE_BEHAVIOR.md`](BLOCK_STATE_FIRE_BEHAVIOR.md).
