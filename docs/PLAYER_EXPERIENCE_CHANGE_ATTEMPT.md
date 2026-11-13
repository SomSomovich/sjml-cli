# Player experience change attempts

`player.experience_change_attempt` runs synchronously on the logical server when
Minecraft is about to apply a player XP delta through
`Player.giveExperiencePoints(int)`. It is backed by NeoForge's cancellable
`PlayerXpEvent.XpChange` event. A callback sees the player's level and progress
before the change and the requested XP amount.

The callback receives only a detached snapshot. It does not receive a live
`Player`, an `ExperienceOrb`, or another Minecraft object. Already-cancelled
events, non-player events, and players absent from the current server session
are skipped. Capture and dispatch must run on the owning server thread; only
dedicated and integrated server modules receive this event.

## Subscription and actions

Subscribe during module initialization and return a synchronous typed action:

```rust
use sjml::{
    event,
    events::{self, EventPriority, PlayerExperienceChangeAction, PlayerExperienceChangeAttempt},
    mod_init, SjmlResult,
};

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("player.experience_change_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.experience_change_attempt")]
fn handle_xp_change(
    event: PlayerExperienceChangeAttempt,
) -> SjmlResult<PlayerExperienceChangeAction> {
    if event.requested_amount > 10 {
        Ok(PlayerExperienceChangeAction::SetAmount(10))
    } else {
        Ok(PlayerExperienceChangeAction::Pass)
    }
}
```

The module must request and the server must grant `events.subscribe` and
`events.cancel` to subscribe. Returning `SetAmount` also requires
`player.vitals.write`.

Actions are resolved in subscription priority and registration order. `Pass`
allows later handlers to decide; the first `Cancel` or authorized `SetAmount`
wins. An unauthorized `SetAmount` is ignored and dispatch continues. `Cancel`
prevents this XP change. `SetAmount(i32)` replaces the requested delta and is
bounded to −1,000,000 through 1,000,000 inclusive. An action decoding error
disables the callback's module and leaves the XP change available to later
handlers.

Callbacks execute synchronously on the server thread. At most 64 subscribed
callbacks are invoked for one XP change. Keep handlers short and avoid waiting
or doing blocking I/O.

## Snapshot and stable callback format

The SDK exposes `PlayerExperienceChangeAttempt` with these fields:

| Field | Type | Meaning |
| --- | --- | --- |
| `player_id` | `String` | Canonical player UUID |
| `level_before` | `i32` | Level before the XP change, from 0 through 20,000 |
| `progress_before` | `f32` | Progress toward the next level, from 0.0 through 1.0 |
| `requested_amount` | `i32` | XP delta requested by Minecraft |

The payload is schema version 1: little-endian `i32` version, length-prefixed
UTF-8 UUID, little-endian `i32` level, little-endian `f32` progress, then
little-endian `i32` requested amount. The UUID is limited to 256 UTF-8 bytes.

The callback action is an unversioned tagged response: byte `0` for `Pass`, byte
`1` for `Cancel`, or byte `2` followed by a little-endian `i32` for
`SetAmount`. The host validates the tag and amount bounds before applying a
decision.
