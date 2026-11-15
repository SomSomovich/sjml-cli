# Entity drown attempts

Server modules can veto a drowning tick before a living entity takes damage:

```rust
use sjml::entity_drown_attempt::EntityDrownAttempt;
use sjml::{event, events::{self, EventPriority}, SjmlResult};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("entity.drown_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.drown_attempt")]
fn keep_zombies_drowning(event: EntityDrownAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:zombie")
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's cancellable `LivingDrownEvent`, fired from `CommonHooks.onLivingBreathe`
once per tick for each living entity whose air supply is exhausted. Returning
`Ok(true)` calls `event.setCanceled(true)`, so the entity neither takes drowning
damage nor loses bubbles or spawns bubble particles this tick. Returning
`Ok(false)` leaves vanilla drowning behavior untouched. Both `events.subscribe`
and `events.cancel` are required, and `#[event_async]` is rejected because the
veto must apply before vanilla reads the event.

The detached schema-v1 payload contains, in order: the canonical entity UUID,
the canonical entity type ID, the dimension ID, the `is_drowning` flag as one
byte, the pending damage amount as little-endian `f32`, and the pending bubble
count as little-endian `i32`. The damage amount must be finite and within 0.0
to 1,000.0; the bubble count must be within 0 to 1,000; strings are bounded to
256 UTF-8 bytes and the payload to 1 KiB. Up to 64 subscribed handlers run per
attempt.

The host ignores events that are already canceled, events captured outside a
`ServerLevel`, events from non-server contexts, and contexts whose dedicated or
integrated mode does not match the owning server. GameTests post a real
`LivingDrownEvent` for a zombie and a cow control and verify that the guest
vetoes only the zombie. Unit tests cover the stable codec, damage and bubble
bounds including `NaN` and infinities, the cancellable NeoForge contract, and
detached snapshot values.
