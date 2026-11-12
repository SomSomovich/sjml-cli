# Player sleep attempts

SJML modules can observe and reject a player's attempt to sleep at a bed through
the synchronous logical-server event `player.sleep_attempt`.

NeoForge 21.1.265 provides `CanPlayerSleepEvent`, fired by
`ServerPlayer.startSleepInBed` after vanilla has checked the attempt. The event
is not an `ICancellableEvent`; it exposes the supported `getProblem` and
`setProblem` result API. SJML listens at the lowest event priority and rejects
an otherwise-allowed attempt by setting `BedSleepingProblem.OTHER_PROBLEM`.
An existing result set before the SJML listener runs is preserved. As with any
NeoForge event result, a later listener at the same priority can still change it.

The event callback receives a detached `PlayerSleepAttempt` with:

- player UUID, profile name, dimension ID, and integer bed position;
- vanilla's original sleep problem and the current result after earlier event
  listeners, when present;
- `can_veto`, which is true only while the current result allows sleeping.

All fields are bounded in schema v1. The guest receives no Minecraft objects.
Callbacks run synchronously on the owning server thread. Subscription requires
`events.subscribe`; cancellation additionally requires `events.cancel` in both
the module manifest and `config/sjml-permissions.toml`.

```rust
use sjml::events::PlayerSleepAttempt;
use sjml::{event, SjmlResult};

#[event(name = "player.sleep_attempt")]
fn sleep_attempt(event: PlayerSleepAttempt) -> SjmlResult<bool> {
    // Preserve vanilla and other mods' existing sleep denial.
    let reject_at_y = event.can_veto && event.bed_position.y < 60;
    Ok(reject_at_y)
}
```

Returning `true` rejects sleep only when `can_veto` is true. The host does not
replace an existing problem or force a player to sleep. The event is a server
API; it has no client-side callback.
