# Player experience-orb pickup attempts

Server modules can veto a player's experience-orb pickup before the orb is
collected:

```rust
use sjml::{
    event,
    events::{self, EventPriority},
    player_xp_pickup_attempt::PlayerXpPickupAttempt,
    SjmlResult,
};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("player.xp_pickup_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.xp_pickup_attempt")]
fn veto_marked_orb_pickup(event: PlayerXpPickupAttempt) -> SjmlResult<bool> {
    Ok(event.orb_value == 7)
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's cancellable `PlayerXpEvent.PickupXp`, fired before the player
collects the orb. Returning `Ok(true)` cancels the event, so the orb stays in
the world and its experience is not added to the player. Returning `Ok(false)`
keeps vanilla behavior.

The detached schema-v1 payload contains the player UUID, the player's dimension
ID, the orb UUID, and the orb value. The value must be within `0..=1_000_000`;
the dimension must be a canonical lowercase resource location. Strings are
bounded to 256 UTF-8 bytes, the payload is capped at 768 bytes, and up to 64
subscribed handlers run per attempt. All subscriptions require both
`events.subscribe` and `events.cancel`. `#[event_async]` is rejected because the
veto must apply while the event is still being dispatched. Events that are
already canceled, a non-server execution context, a player outside a
`ServerLevel`, or a dedicated/integrated context mismatch are not dispatched.

GameTests post a real `PlayerXpEvent.PickupXp` for an orb with value seven and
verify that it is canceled. Unit tests cover the exact schema-v1 bytes, orb
value bounds, the pinned schema version, the cancellable NeoForge contract, and
detached snapshot values.
