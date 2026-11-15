# Player permission change attempts

Server modules can reject a player's operator permission-level change before
NeoForge applies it:

```rust
use sjml::{
    event,
    events::{self, EventPriority},
    player_permissions_change_attempt::PlayerPermissionsChangeAttempt,
    SjmlResult,
};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("player.permissions_change_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.permissions_change_attempt")]
fn veto_operator_promotion(event: PlayerPermissionsChangeAttempt) -> SjmlResult<bool> {
    Ok(event.new_level == 4)
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's cancellable `PermissionsChangedEvent`, which vanilla fires when a
player is opped or deopped. Returning `Ok(true)` cancels the event, so the
operator change is rejected and the player keeps their current permission
level. Returning `Ok(false)` allows the requested level to be applied.

The detached schema-v1 payload contains the player UUID, the player's dimension
ID, the requested permission level, and the level the player had before the
change. Both levels must be within `0..=4`. IDs must be canonical lowercase
resource locations; strings are bounded to 256 UTF-8 bytes, the payload is
capped at 512 bytes, and up to 64 subscribed handlers run per attempt. All
subscriptions require both `events.subscribe` and `events.cancel`.
`#[event_async]` is rejected because the veto must be applied while the event
is still being dispatched. Events with a canceled state, a non-server execution
context, a player outside a `ServerLevel`, or a dedicated/integrated context
mismatch are not dispatched.

GameTests post a real `PermissionsChangedEvent` for a level-four promotion and
a non-promotion control, verifying that only the promotion is canceled. Unit
tests cover the exact schema-v1 bytes, level bounds, the pinned schema version,
the cancellable NeoForge contract, and detached snapshot values.
