# Feature growth attempts

Server modules can veto a block growing into a configured feature, such as a
sapling or a bonemeal-driven tree, before any blocks are placed:

```rust
use sjml::{block_grow_feature_attempt::BlockGrowFeatureAttempt, event, events::{self, EventPriority}, SjmlResult};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("world.block_grow_feature_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "world.block_grow_feature_attempt")]
fn stop_growth_without_a_feature(event: BlockGrowFeatureAttempt) -> SjmlResult<bool> {
    Ok(event.feature_id.is_none())
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's cancellable `BlockGrowFeatureEvent`. Returning `Ok(true)` calls
`event.setCanceled(true)`, so the feature is not placed and the original block
stays in place. Returning `Ok(false)` keeps vanilla behavior.

The detached schema-v1 payload contains the dimension ID, the growing block
x/y/z, and an optional canonical registry ID for the configured feature. The
feature is encoded as a `u8` presence flag followed by the feature string only
when the flag is `1`; `event.getFeature()` may be absent, and an unreferenced
`Holder` also yields no feature ID, so Rust decodes both cases as
`Option<String>::None`. The position is bounded to ±30,000,000 in x/z; strings
are bounded to 256 UTF-8 bytes and the payload to 1 KiB. Up to 64 subscribed
handlers run per attempt, and all subscriptions require both `events.subscribe`
and `events.cancel`. `#[event_async]` is rejected because the veto must apply
before the feature places blocks.

GameTests post a real `BlockGrowFeatureEvent` with an absent feature and verify
that the guest veto cancels it. Unit tests cover the stable codec with and
without a feature, bounds, the cancellable NeoForge contract, and detached
snapshot values.
