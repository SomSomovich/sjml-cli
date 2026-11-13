# Player experience level change attempts

`player.experience_level_change_attempt` runs synchronously on the logical
server when Minecraft is about to apply a level delta through NeoForge's
cancellable `PlayerXpEvent.LevelChange`. The snapshot reports the player's
level before the change and the requested number of levels.

Callbacks receive only detached values: the canonical player UUID, the
pre-change level, and the requested delta. No `Player`, `ExperienceOrb`, or
other Minecraft object crosses the module boundary. Already-cancelled events,
non-player events, and players absent from the current server session are
skipped. Capture and dispatch run on the owning server thread, and only
dedicated or integrated server modules receive this event.

## Subscribe and choose an action

Subscribe during module initialization and return a synchronous typed action:

```rust
use sjml::{
    event,
    events::{self, EventPriority, PlayerExperienceLevelChangeAction,
        PlayerExperienceLevelChangeAttempt},
    mod_init, SjmlResult,
};

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("player.experience_level_change_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.experience_level_change_attempt")]
fn handle_levels(
    event: PlayerExperienceLevelChangeAttempt,
) -> SjmlResult<PlayerExperienceLevelChangeAction> {
    if event.requested_levels == 5 {
        Ok(PlayerExperienceLevelChangeAction::SetLevels(2))
    } else {
        Ok(PlayerExperienceLevelChangeAction::Pass)
    }
}
```

The module must request and the server must grant `events.subscribe` and
`events.cancel` to subscribe. Returning `SetLevels` also requires
`player.vitals.write`.

Actions run in subscription priority and registration order. `Pass` lets later
handlers decide; the first `Cancel` or authorized `SetLevels` wins. An
unauthorized `SetLevels` is ignored and dispatch continues. `Cancel` prevents
the requested level change. `SetLevels(i32)` replaces its delta and is bounded
to −20,000 through 20,000 inclusive. Invalid callback output disables the
module and lets later handlers run.

Callbacks run synchronously on the server thread. At most 64 subscribed
callbacks are invoked for one event. Keep handlers short and avoid waiting or
blocking I/O.

## Snapshot and wire format

The SDK exposes `PlayerExperienceLevelChangeAttempt`:

| Field | Type | Meaning |
| --- | --- | --- |
| `player_id` | `String` | Canonical player UUID |
| `level_before` | `i32` | Player level before this change, bounded to 0 through 1,000,000 |
| `requested_levels` | `i32` | Level delta requested by Minecraft |

The schema-v1 payload contains a little-endian `i32` version, a length-prefixed
UTF-8 UUID, then little-endian `i32` values for `level_before` and
`requested_levels`. The UUID is limited to 256 UTF-8 bytes.

The callback action is unversioned: byte `0` means `Pass`, byte `1` means
`Cancel`, and byte `2` followed by a little-endian `i32` means `SetLevels`. The
host validates the tag and the −20,000 through 20,000 bound before applying an
action.
