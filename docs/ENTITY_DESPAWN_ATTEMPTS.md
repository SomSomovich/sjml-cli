# Mob despawn attempts

Server modules can veto a natural mob despawn before Minecraft removes the mob:

```rust
use sjml::{
    event,
    events::{self, EventPriority},
    mod_init, EntityDespawnAttempt, SjmlResult,
};

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.despawn_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.despawn_attempt")]
fn keep_zombies_alive(event: EntityDespawnAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:zombie")
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's `MobDespawnEvent`, fired from `Mob.checkDespawn()` once per tick for
each mob that attempts to despawn. Returning `Ok(true)` sets the event result
to `DENY`, which keeps the mob alive: Minecraft does not remove it, drop loot,
or award experience. Returning `Ok(false)` leaves `DEFAULT`, so vanilla despawn
logic applies. `MobDespawnEvent` is not cancellable, so the host maps the guest
veto to `setResult(DENY)` instead of canceling the event.

The detached schema-v1 payload contains the canonical mob UUID, the entity type
ID, the dimension ID, and the mob's x/y/z position as little-endian `f64`
values. Coordinates must be finite and within ±30,000,000; strings are bounded
to 256 UTF-8 bytes and the payload to 1 KiB. Up to 64 subscribed handlers run
per attempt, and all subscriptions require both `events.subscribe` and
`events.cancel`. `#[event_async]` is rejected because the result must be set
before `checkDespawn()` reads it.

The host ignores events whose result is already non-`DEFAULT`, events from
non-server contexts, and mobs that are not in a `ServerLevel`; dedicated and
integrated server contexts must match. Unit tests cover the stable codec,
coordinate and ID bounds, the NeoForge result contract, and detached snapshot
values. GameTests post a real `MobDespawnEvent` for a zombie and a cow control
and verify that the guest veto produces `DENY` only for the zombie.
