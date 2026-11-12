# Cancellable entity damage attempts

`entity.damage_attempt` runs synchronously on the logical server while an incoming hit is being
processed. Return `Ok(true)` to veto the damage or `Ok(false)` to let NeoForge continue:

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.damage_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.damage_attempt")]
fn protect_players(event: EntityDamageAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:player")
}
```

The module must request and receive both `events.subscribe` and `events.cancel`. The handler must
use synchronous `#[event]`; `#[event_async]` is rejected because its return value controls the
live NeoForge event. Any authorized handler returning `true` vetoes the full damage sequence. A
callback trap or malformed result disables only that module and does not cancel the hit. An event
already canceled by another listener is not dispatched.

For NeoForge 21.1.265, the host listens to `LivingIncomingDamageEvent`. Its API marks it
cancellable, and `CommonHooks.onEntityIncomingDamage` returns the event's canceled state to the
damage pipeline. NeoForge fires it after invulnerability checks and before damage reductions such
as armor are applied. Consequently this event cannot observe attacks that failed those earlier
checks. Its `original_damage` is the amount passed into the damage sequence, while `amount` is the
current pre-mitigation value when SJML captures the event.

`EntityDamageAttempt` contains the victim UUID, entity type, dimension, damage type, optional
responsible source entity UUID, finite coordinates, and the original/current damage amounts. It
contains detached values only; no Minecraft object crosses the WASM boundary. The callback runs
on the owning logical-server thread in both dedicated and integrated servers.

Schema v1 encodes the version, four UTF-8 strings (victim UUID, entity type, dimension, and damage
type), an optional source UUID, three little-endian `f64` coordinates, and two little-endian `f32`
damage amounts. Strings are limited to 65,536 bytes and the payload to 1 MiB. Coordinates and
amounts must be finite; damage amounts must be non-negative.
