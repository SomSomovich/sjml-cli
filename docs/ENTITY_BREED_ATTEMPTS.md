# Cancellable entity breeding attempts

`entity.breed_attempt` runs synchronously when NeoForge is about to add a proposed animal
offspring. Return `Ok(true)` to cancel that child spawn:

```rust
use sjml::{event, events::{self, EntityBreedAttempt, EventPriority}, mod_init, SjmlResult};

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.breed_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.breed_attempt")]
fn prevent_cow_babies(event: EntityBreedAttempt) -> SjmlResult<bool> {
    Ok(event.parent_a_type_id == "minecraft:cow"
        && event.parent_b_type_id == "minecraft:cow")
}
```

The module must request and receive both `events.subscribe` and `events.cancel`. A veto cancels
the proposed offspring and NeoForge's breeding hook does not add the child. It does not rewind the
parents' completed breeding interaction or restore their ability to mate immediately.

The host listens to NeoForge `BabyEntitySpawnEvent` at the lowest event priority and skips an
event already canceled by another listener. It captures the event once on the owning logical-server
thread and routes the synchronous callback through the subscribing module's owner context. The
callback can veto but cannot replace the proposed child. A callback trap or malformed result
disables only that module and allows vanilla handling to continue.

`EntityBreedAttempt` contains detached UUID strings for both parents and the proposed child, the
registry entity-type IDs for those three entities, and the dimension ID. It contains no live
Minecraft references. The host limits each UTF-8 field to 256 bytes and the complete schema-v1
payload to 2 KiB. The event is server-only and cannot use `#[event_async]` because its boolean
result controls the live NeoForge event.

Schema v1 is little-endian: `u32` version `1`, then seven stable UTF-8 strings in this order:
`parent_a_id`, `parent_a_type_id`, `parent_b_id`, `parent_b_type_id`, `child_id`,
`child_type_id`, `dimension_id`. Each string is encoded as an `i32` byte length followed by its
UTF-8 bytes. Entity type and dimension values are canonical lowercase resource locations.

The dedicated-server GameTest `EntityBreedAttemptGameTests` calls the real animal breeding hook,
verifies the guest receives one detached snapshot, and checks that the proposed calf is not added.
