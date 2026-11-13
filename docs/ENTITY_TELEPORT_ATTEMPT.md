# Entity teleport attempts

`entity.teleport_attempt` is a synchronous, cancellable logical-server event
for NeoForge's `EntityTeleportEvent` family. It exposes the live entity UUID,
entity type, current dimension, stable cause code, and detached source and
destination coordinates. The callback may pass or cancel the underlying
teleport before Minecraft applies it.

Subscribe in a module with `events.subscribe` and `events.cancel` and declare a
synchronous handler:

```rust
use sjml::{
    event,
    events::{self, EventPriority},
    mod_init, EntityTeleportAttempt, EntityTeleportAttemptAction, SjmlResult,
};

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.teleport_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.teleport_attempt")]
fn guard_teleport(event: EntityTeleportAttempt) -> SjmlResult<EntityTeleportAttemptAction> {
    if event.target_x.abs() > 1_000_000.0 {
        Ok(EntityTeleportAttemptAction::Cancel)
    } else {
        Ok(EntityTeleportAttemptAction::Pass)
    }
}
```

The `cause` field is one of `Command`, `SpreadPlayersCommand`, `EnderEntity`,
`EnderPearl`, `ChorusFruit`, or `Other`. The event covers NeoForge's documented
server teleport hooks for `/tp`, `/spreadplayers`, Enderman/Shulker random
teleports, Ender Pearls, and Chorus Fruit, plus subclasses posted by mods. It
does not observe every direct call to `Entity.teleportTo`, cross-dimension
portal transfer, or SJML's own entity-teleport import. Client-side events are
ignored.

The payload uses schema 1: three length-prefixed UTF-8 strings (canonical entity
UUID, entity type ID, and dimension ID), a little-endian `u32` cause code, then
six little-endian `f64` values (`from_x/y/z`, `target_x/y/z`). Cause codes are
`0=COMMAND`, `1=SPREAD_PLAYERS_COMMAND`, `2=ENDER_ENTITY`, `3=ENDER_PEARL`,
`4=CHORUS_FRUIT`, and `5=OTHER`. IDs are limited
to 256 UTF-8 bytes, and coordinates must be finite and within ±30,000,000.
The response is one byte: `0` passes, `1` cancels. At most 64 authorized
callbacks run for one attempt, serially on the owning server thread. The event
is canceled if a callback returns `Cancel`.

The host ignores client, pre-cancelled, removed-entity, and unloaded-source
events. It only checks whether the source chunk is already loaded and does not
load the destination chunk. The callback runs on the owning server thread; no
live Minecraft object or handle crosses the ABI. The event is an observation
and veto hook and does not authorize the guest to initiate teleportation.
