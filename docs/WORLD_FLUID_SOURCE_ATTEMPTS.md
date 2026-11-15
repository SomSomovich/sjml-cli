# Fluid source attempts

Server modules can veto infinite-source formation before a fluid converts:

```rust
use sjml::world_fluid_source_attempt::{WorldFluidSourceAttempt, EVENT_NAME};
use sjml::{
    event,
    events::{self, EventPriority},
    mod_init, SjmlResult,
};

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe(EVENT_NAME, EventPriority::Normal)?;
    Ok(())
}

#[event(name = "world.fluid_source_attempt")]
fn veto_water_source_formation(event: WorldFluidSourceAttempt) -> SjmlResult<bool> {
    Ok(event.fluid_id == "minecraft:water")
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's `CreateFluidSourceEvent`, fired when a flowing fluid checks nearby
blocks that would convert it into a source block. Returning `Ok(true)` calls
`setCanConvert(false)`, so the infinite source never forms. Returning `Ok(false)`
leaves the vanilla decision untouched. `CreateFluidSourceEvent` is not an
`ICancellableEvent`, so the host maps the guest veto to `setCanConvert(false)`
instead of canceling the event.

The detached schema-v1 payload contains the canonical dimension ID, the fluid
block x/y/z as little-endian `i32` values, the registry ID of
`event.getFluidState().getType()` (for example `minecraft:water`), and
`vanilla_result`, the precomputed value of `event.getVanillaResult()`. The x/z
coordinates are bounded to ±30,000,000; strings are bounded to 256 UTF-8 bytes
and the payload to 1 KiB. Up to 64 subscribed handlers run per attempt, and all
subscriptions require both `events.subscribe` and `events.cancel`.
`#[event_async]` is rejected because the flag must be set before
`CreateFluidSourceEvent` returns.

The host ignores events from non-server contexts, non-`ServerLevel` levels, and
mismatched dedicated/integrated contexts, and it rejects capture off the owning
server thread. Unit tests cover the stable codec, coordinate and ID bounds, the
NeoForge contract, and detached snapshot values. GameTests post a real
`CreateFluidSourceEvent` for water and a lava control and verify that only the
water formation is vetoed.
