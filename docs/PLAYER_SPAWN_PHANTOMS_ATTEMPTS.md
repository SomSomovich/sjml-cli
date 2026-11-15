# Phantom spawn attempts

Server modules can veto a phantom spawn before Minecraft picks a spawn
position for the group:

```rust
use sjml::{
    event,
    events::{self, EventPriority},
    mod_init, PlayerSpawnPhantomsAttempt, SjmlResult,
};

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("player.spawn_phantoms_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.spawn_phantoms_attempt")]
fn deny_large_phantom_groups(event: PlayerSpawnPhantomsAttempt) -> SjmlResult<bool> {
    Ok(event.phantoms_to_spawn >= 3)
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's `PlayerSpawnPhantomsEvent`, fired once per player from
`PhantomSpawner.tick()` when the vanilla spawn conditions would attempt a
group of phantoms. Returning `Ok(true)` sets the event result to `DENY`, which
suppresses the spawn attempt entirely for that player. Returning `Ok(false)`
leaves `DEFAULT`, so the vanilla sky, difficulty, and rest-time checks apply.
`PlayerSpawnPhantomsEvent` is not cancellable, so the host maps the guest veto
to `setResult(DENY)` instead of canceling the event.

The detached schema-v1 payload contains the canonical player UUID, the
dimension ID, the pending phantom count (`i32`, in the range 0..=64), and the
player's x/y/z position as little-endian `f64` values. Coordinates must be
finite and within ±30,000,000; strings are bounded to 256 UTF-8 bytes and the
payload to 1 KiB. Up to 64 subscribed handlers run per attempt, and all
subscriptions require both `events.subscribe` and `events.cancel`.
`#[event_async]` is rejected because the result must be set before
`PhantomSpawner` reads it.

The host ignores events whose result is already non-`DEFAULT`, events from
non-server contexts, and players that are not in a `ServerLevel`; dedicated
and integrated server contexts must match. Unit tests cover the stable codec,
phantom-count and coordinate bounds, the NeoForge result contract, and
detached snapshot values. GameTests post a real `PlayerSpawnPhantomsEvent`
with three phantoms and a one-phantom control, then verify that the guest veto
produces `DENY` only for the marked count.
