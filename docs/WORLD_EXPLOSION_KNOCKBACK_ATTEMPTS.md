# Explosion knockback attempts

Server modules can veto the knockback an explosion applies to an entity:

```rust
use sjml::{
    event,
    events::{self, EventPriority},
    world_explosion_knockback_attempt::WorldExplosionKnockbackAttempt,
    SjmlResult,
};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("world.explosion_knockback_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "world.explosion_knockback_attempt")]
fn stop_zombie_knockback(event: WorldExplosionKnockbackAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:zombie")
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's `ExplosionKnockbackEvent`, which Minecraft posts once an explosion has
calculated the velocity to add to an entity caught in the blast. The NeoForge
event is not itself cancellable; returning `Ok(true)` makes the host call
`setKnockbackVelocity(Vec3.ZERO)`, so the entity receives no explosion impulse.
Returning `Ok(false)` keeps the calculated velocity. The event only fires for
integrated or dedicated server contexts, and a capture attempted off the owning
thread is rejected.

The detached schema-v1 payload contains the dimension ID, the affected entity's
canonical UUID and registry type, the number of blocks the explosion would
affect (`0..=4,096`), and the knockback velocity `vx`, `vy`, `vz` in blocks per
tick. Each velocity axis must be finite and within `±1,000`. IDs must be
canonical lowercase resource locations; strings are bounded to 256 UTF-8 bytes,
the payload to 1 KiB, and up to 64 subscribed handlers run per attempt. All
subscriptions require both `events.subscribe` and `events.cancel`.
`#[event_async]` is rejected because the veto must apply before vanilla adds the
velocity to the entity.

The compilable SDK example is
`sjml-cli/sjml-sdk/examples/gametest/world_explosion_knockback_attempt.rs`.

GameTests spawn a zombie, create a real explosion at the test origin, post
`ExplosionKnockbackEvent` for the zombie, and verify that the guest zeroes the
knockback velocity. Unit tests cover the stable codec, finite and bound checks,
the non-cancellable NeoForge contract, and detached snapshot values.
