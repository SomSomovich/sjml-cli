# Spawn-point attempts

Server modules can veto a player's respawn-point change before Minecraft applies
it:

```rust
use sjml::{event, events::{self, EventPriority, PlayerSpawnPointAttempt}, SjmlResult};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("player.spawn_point_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.spawn_point_attempt")]
fn keep_spawns_in_overworld(event: PlayerSpawnPointAttempt) -> SjmlResult<bool> {
    Ok(event.dimension_id != "minecraft:overworld" && !event.forced)
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's cancellable `PlayerSetSpawnEvent`. Returning `Ok(true)` cancels the
change; returning `Ok(false)` leaves the vanilla result. It covers bed and
respawn-anchor use as well as forced server-side changes, which are marked by
the `forced` flag.

The detached schema-v1 payload contains the player UUID, the target spawn
dimension, an optional block position (absent when the point is being cleared),
and the forced flag. The position is bounded to ±30,000,000 on every axis; IDs
must be canonical lowercase resource locations. No Minecraft objects or player
handles are exposed. Up to 64 subscribed handlers run per attempt, and all
subscriptions require both `events.subscribe` and `events.cancel`.
`#[event_async]` is rejected because the veto decision must be applied before
the spawn hook returns.

GameTests post a real `PlayerSetSpawnEvent` for a mock server player and verify
that the guest veto cancels only the non-forced change while a forced change
keeps its vanilla result. Unit tests cover both payload variants, bounds, the
cancellable NeoForge contract, and detached snapshot values.
