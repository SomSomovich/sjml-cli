# Entity lightning-strike attempts

Server modules can veto lightning damage against one specific entity:

```rust
use sjml::{event, events::{self, EntityLightningStrikeAttempt, EventPriority}, SjmlResult};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("entity.lightning_strike_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.lightning_strike_attempt")]
fn protect_pigs(event: EntityLightningStrikeAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:pig")
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's cancellable `EntityStruckByLightningEvent`. Returning `Ok(true)`
skips the strike for that entity only; the bolt still exists and can affect
other entities, blocks, and lightning rods. Returning `Ok(false)` leaves the
vanilla behavior unchanged.

The detached schema-v1 payload contains the target entity UUID and registry
type, the dimension ID, the bolt UUID, and the bolt's `f64` x/y/z position.
Coordinates must be finite and within ±30,000,000; IDs must be canonical
lowercase resource locations; strings are bounded to 256 UTF-8 bytes and the
payload to 2 KiB. Up to 64 subscribed handlers run per attempt, and all
subscriptions require both `events.subscribe` and `events.cancel`.
`#[event_async]` is rejected because the veto decision must be applied before
the strike returns.

GameTests post a real `EntityStruckByLightningEvent` for a spawned pig and cow
and verify that the guest veto blocks only the marked pig strike while the
unmarked cow strike keeps its vanilla result. Unit tests cover the stable codec,
bounds, the cancellable NeoForge contract, and detached snapshot values.
