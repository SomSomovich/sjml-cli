# Entity dimension travel attempts

Server modules can veto a dimension change before the entity leaves its level:

```rust
use sjml::{
    entity_travel_dimension_attempt::EntityTravelDimensionAttempt, event,
    events::{self, EventPriority}, SjmlResult,
};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("entity.travel_dimension_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.travel_dimension_attempt")]
fn block_overworld_to_nether(event: EntityTravelDimensionAttempt) -> SjmlResult<bool> {
    Ok(event.from_dimension_id == "minecraft:overworld"
        && event.to_dimension_id == "minecraft:the_nether")
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's cancellable `EntityTravelToDimensionEvent`. Returning `Ok(true)`
prevents the transfer entirely: the entity keeps its current level, position,
and passengers. Returning `Ok(false)` keeps vanilla behavior.

The detached schema-v1 payload contains the entity's canonical UUID, its
registry type ID, the source dimension ID, and the requested destination
dimension ID. Every ID must be a canonical lowercase resource location;
strings are bounded to 256 UTF-8 bytes and the payload to 1 KiB. Up to 64
subscribed handlers run per attempt, and all subscriptions require both
`events.subscribe` and `events.cancel`. `#[event_async]` is rejected because
the veto must apply before vanilla moves the entity. Events that are already
canceled, client-side contexts, and entities outside a server level are never
dispatched to guests.

GameTests post a real `EntityTravelToDimensionEvent` for an overworld player
traveling to the nether and an overworld control, verifying that only the
marked transfer is canceled. Unit tests cover the stable codec, bounds, the
cancellable NeoForge contract, and detached snapshot values.
