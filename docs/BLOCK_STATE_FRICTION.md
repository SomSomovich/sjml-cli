# Per-state block friction

Registered blocks can override movement friction for an exact declared block state with
`registry::set_block_state_behavior`:

```rust
registry::set_block_state_behavior(
    "ice_block",
    &[("slippery", "true")],
    registry::BlockStateBehavior::new().friction(0.98)?,
)?;
```

Declare every state property before setting an override. The selector must name each property once
with a canonical value. Friction must be finite and in `0.0..=2.0`; other states keep the
block-wide friction from `BlockBehavior` (default `0.6`). Multiple calls may add disjoint state
behavior fields, while assigning the same field twice to one state is rejected.
Call this from `#[mod_register]` in a module that declares the block and requests `content.register`.

The host uses NeoForge's `IBlockExtension.getFriction(BlockState, LevelReader, BlockPos, Entity)`
hook. Minecraft 1.21.1 does not offer equivalent state-aware hooks for speed and jump factors;
`BlockBehavior::speed_factor` and `BlockBehavior::jump_factor` therefore remain block-wide.
Guests cannot run during friction queries: values are resolved into immutable, bounded tables
before block registration.

The behavior payload schema has since grown additively through v5. The host continues to accept
schema-v1 through v4 payloads for existing guests.
