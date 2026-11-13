# Living-entity fall attempts

The synchronous, logical-server `entity.fall_attempt` event runs through
NeoForge 21.1.265's cancellable `LivingFallEvent`. A handler returning `Ok(true)`
cancels fall damage before Minecraft applies it. A `false` result preserves the
current event result. The callback does not change fall distance or the damage
multiplier.

```rust
use sjml::{event, events::{self, EntityFallAttempt, EventPriority}, SjmlResult};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("entity.fall_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.fall_attempt")]
fn cushion_named_entity(event: EntityFallAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:chicken")
}
```

The detached schema-v1 payload contains the entity UUID, entity type and
dimension IDs, position, fall distance, and current damage multiplier. It
contains no Minecraft object references. IDs, coordinates, and floating-point
values are bounded by the ABI codec. Up to 64 subscribed handlers run
synchronously on the owning server thread; all subscriptions require both
`events.subscribe` and `events.cancel`. `#[event_async]` is rejected for this
event because the cancellation decision must be applied before the fall hook
returns.

The GameTest calls NeoForge's `CommonHooks.onLivingFall` with a spawned pig and
verifies that the guest veto prevents fall damage. Unit tests cover the event
schema, value bounds, detached payload, subscription grants, and host event
contract.
