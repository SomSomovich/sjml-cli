# Entity mount and dismount attempts

The synchronous logical-server `entity.mount_attempt` event is backed by
NeoForge 21.1.265's cancellable `EntityMountEvent`. It fires before Minecraft
changes an entity's riding relationship. A handler returning `Ok(true)` cancels
the attempted mount or dismount; `false` preserves the event's current result.

```rust
use sjml::{event, events::{self, EntityMountAttempt, EventPriority}, SjmlResult};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("entity.mount_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.mount_attempt")]
fn keep_pigs_off_minecarts(event: EntityMountAttempt) -> SjmlResult<bool> {
    let targets_minecart = event.mounted_entity.as_ref()
        .is_some_and(|target| target.entity_type_id == "minecraft:minecart");
    Ok(event.mounting
        && event.entity_type_id == "minecraft:pig"
        && targets_minecart)
}
```

The schema-v1 snapshot carries the actor UUID, actor type ID, dimension ID, an
optional target UUID/type pair, and a boolean that is true for mounting. The
target is optional because NeoForge documents that `getEntityBeingMounted()`
may return `null`. The snapshot contains no Minecraft object references. UUIDs
and resource locations use canonical text forms, each string is capped at 256
UTF-8 bytes, and the full payload is capped at 2 KiB.

The wire layout is little-endian `u32(1)`, length-prefixed `entity_id`,
`entity_type_id`, and `dimension_id`, a one-byte option tag, then—when present—
the target's nested schema `u32(1)` followed by length-prefixed target UUID and
type ID. A final byte encodes `mounting` as 0 or 1. At most 64 subscribed
handlers run synchronously on the owning server thread. Both `events.subscribe`
and `events.cancel` are required; `#[event_async]` is rejected for this event.

`EntityMountAttemptGameTests` makes a pig call Minecraft's real
`startRiding(minecart)` path and verifies that the guest veto prevents a riding
relationship. A second GameTest mounts a cow, then vetoes its `stopRiding()`
attempt and verifies the existing riding relationship remains. Unit tests cover
both optional-target wire forms, detached values, adapter event contract, and
subscription capabilities.
