# Cancellable entity death attempts

`entity.death_attempt` is a synchronous logical-server callback for a living
entity death that NeoForge is about to accept. Return `Ok(true)` to cancel the
death or `Ok(false)` to allow it:

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.death_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.death_attempt")]
fn protect_players(event: EntityDeathAttempt) -> SjmlResult<bool> {
    Ok(event.player_display_name.is_some())
}
```

The module must request and receive both `events.subscribe` and
`events.cancel`. Handlers run on the server thread while NeoForge processes
`LivingDeathEvent`; they cannot use `#[event_async]`. Any authorized `true`
result cancels the attempt. A failed or malformed callback disables only its
module and does not cancel the death. An attempt already canceled by another
handler is not dispatched.

When an authorized handler vetoes the event, the host restores the victim to at
least one health. NeoForge raises `LivingDeathEvent` after health reaches zero,
so canceling only the event would otherwise leave the entity in a zero-health
state. The original `entity.death` notification is skipped for the vetoed
attempt.

`EntityDeathAttempt` contains the victim UUID, entity type, dimension, finite
coordinates, and an optional player profile name. The optional name is present
only when the victim is a server player. It carries detached values only; no
Minecraft entity or player object crosses the WASM boundary.

When SJML allows a new death attempt, the existing non-cancellable
`entity.death` notification is dispatched with its original `EntityDeath`
schema and `events.subscribe` requirement. An event already canceled by another listener is
not offered as an attempt; the existing `entity.death` observer behavior for
that event remains unchanged.

The schema-v1 attempt payload contains the little-endian version, stable UTF-8
strings for UUID, type ID, and dimension ID, an optional string for the player
profile name, then three little-endian finite `f64` coordinates. Each string is
limited to 65,536 bytes and the complete payload to 1 MiB.
