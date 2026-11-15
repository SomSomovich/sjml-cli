# Entity finalize-spawn attempts

Server modules can veto a mob spawn before NeoForge finalizes its initial state:

```rust
use sjml::entity_finalize_spawn_attempt::EntityFinalizeSpawnAttempt;
use sjml::{event, events::{self, EventPriority}, SjmlResult};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("entity.finalize_spawn_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.finalize_spawn_attempt")]
fn block_zombie_spawns(attempt: EntityFinalizeSpawnAttempt) -> SjmlResult<bool> {
    Ok(attempt.entity_type_id == "minecraft:zombie")
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's cancellable `FinalizeSpawnEvent`. Returning `Ok(true)` calls
`event.setSpawnCancelled(true)`, so the entity is never added to the level.
Returning `Ok(false)` leaves vanilla behavior untouched. The host never calls
`setCanceled`, because cancelling the event only skips `Mob.finalizeSpawn` and
still lets the entity spawn.

The detached schema-v1 payload contains, in order: the canonical entity UUID,
the canonical entity type ID, the dimension ID, entity x/y/z as `f64`, the
lowercase `MobSpawnType` name, and the effective difficulty as `f32`.
Coordinates must be finite and within ±30,000,000; the difficulty must be
finite and within 0.0 to 1,000,000.0; strings are bounded to 256 UTF-8 bytes
(the spawn type to 64) and the payload to 2 KiB. Up to 64 subscribed handlers
run per attempt, and all subscriptions require both `events.subscribe` and
`events.cancel`. `#[event_async]` is rejected because the veto must apply
before the entity is spawned.

Captures already marked as spawn-cancelled are ignored so multiple veto
handlers do not re-dispatch the same attempt. GameTests finalize a real zombie
and cow through `Mob.finalizeSpawn` and verify that only the marked zombie is
spawn-cancelled. Unit tests cover the stable codec, numeric and identifier
bounds, the cancellable NeoForge contract, and detached snapshot values.
