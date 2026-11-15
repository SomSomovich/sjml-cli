# Player arrow-nock attempts

Server modules can veto a bow or crossbow draw before the use action starts:

```rust
use sjml::{
    event,
    events::{self, EventPriority},
    player_arrow_nock_attempt::PlayerArrowNockAttempt,
    SjmlResult,
};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("player.arrow_nock_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.arrow_nock_attempt")]
fn veto_vanilla_bows(event: PlayerArrowNockAttempt) -> SjmlResult<bool> {
    Ok(event.item_id == "minecraft:bow" && event.has_ammo)
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's cancellable `ArrowNockEvent`. Returning `Ok(true)` cancels the event,
so the use action fails and the player cannot begin drawing. Returning
`Ok(false)` keeps vanilla behavior. Already-canceled events and callbacks
observed off the server thread are not dispatched.

The detached schema-v1 payload contains the player's canonical UUID, the
dimension ID, the lowercase hand name (`main_hand` or `off_hand`), the held
item's canonical ID (`minecraft:air` when the hand is empty), the stack count
(0 through 1,000,000), and a `has_ammo` flag. Strings are bounded to 256 UTF-8
bytes and the payload to 1 KiB. Up to 64 subscribed handlers run per attempt,
and every subscription requires both `events.subscribe` and `events.cancel`.
`#[event_async]` is rejected because the veto must apply before the draw
starts.

GameTests post a real `ArrowNockEvent` for a bow and a stick control and verify
that only the bow draw is canceled. Unit tests cover the stable codec, hand
vocabulary, bounds, the cancellable NeoForge contract, and detached snapshot
values.
