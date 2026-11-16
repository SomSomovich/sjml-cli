# Player game-mode change attempts

Server modules can reject a player's game-mode change before NeoForge applies
it:

```rust
use sjml::{
    event,
    events::{self, EventPriority},
    player_game_mode_change_attempt::PlayerGameModeChangeAttempt,
    SjmlResult,
};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("player.game_mode_change_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.game_mode_change_attempt")]
fn veto_creative_mode(event: PlayerGameModeChangeAttempt) -> SjmlResult<bool> {
    Ok(event.new_game_mode == "creative")
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's cancellable `PlayerEvent.PlayerChangeGameModeEvent`, which vanilla
fires before the requested mode is applied. Returning `Ok(true)` cancels the
event, so the game-mode change is rejected and the player keeps their current
mode. Returning `Ok(false)` allows the requested mode to be applied.

The detached schema-v1 payload contains the player UUID, the player's dimension
ID, the current game mode, and the requested game mode. Game-mode names are
lowercase vanilla values: `survival`, `creative`, `adventure`, or `spectator`.
IDs must be canonical lowercase resource locations; strings are bounded to 256
UTF-8 bytes, the payload is capped at 768 bytes, and up to 64 subscribed
handlers run per attempt. All subscriptions require both `events.subscribe` and
`events.cancel`. `#[event_async]` is rejected because the veto must be applied
while the event is still being dispatched. Events with a canceled state, a
non-server execution context, a player outside a `ServerLevel`, or a
dedicated/integrated context mismatch are not dispatched.

GameTests post a real `PlayerChangeGameModeEvent` from survival to creative and
a creative-to-survival control, verifying that only the creative transition is
canceled. Unit tests cover the exact schema-v1 bytes, the game-mode vocabulary,
the pinned schema version, the cancellable NeoForge contract, and detached
snapshot values.
