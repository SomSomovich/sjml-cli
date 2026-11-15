# Player arrow loose attempts

Server modules can veto a bow or crossbow release before the projectile spawns:

```rust
use sjml::{
    event,
    events::{self, EventPriority},
    player_arrow_loose_attempt::PlayerArrowLooseAttempt,
    SjmlResult,
};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("player.arrow_loose_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.arrow_loose_attempt")]
fn block_fully_charged_bow(event: PlayerArrowLooseAttempt) -> SjmlResult<bool> {
    Ok(event.item_id == "minecraft:bow" && event.charge == 20 && event.has_ammo)
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's cancellable `ArrowLooseEvent`. Returning `Ok(true)` cancels the
event: no arrow or crossbow bolt is fired and the player keeps holding the bow.
Returning `Ok(false)` leaves the release unchanged. Crossbows report a charge
of 1 in vanilla, while a fully drawn bow reports 20.

Only releases by server players inside a server level are dispatched.
Already-canceled events, non-server contexts, and dedicated/integrated context
mismatches are ignored, and capturing off the owning server thread throws
`IllegalStateException`.

The detached schema-v1 payload contains the player UUID, the dimension ID, the
released item ID, the stack count (0 to 1,000,000), the charge (0 to 10,000),
and a `has_ammo` flag. IDs must be canonical lowercase resource locations,
strings are bounded to 256 UTF-8 bytes, and the payload is capped at 1 KiB. Up
to 64 subscribed handlers run per attempt, and all subscriptions require both
`events.subscribe` and `events.cancel`. `#[event_async]` is rejected because the
veto must apply before the projectile entity is created.

GameTests post a real `ArrowLooseEvent` for a fully charged bow and a
partially charged control and verify that only the marked release is canceled.
Unit tests cover the stable codec, charge and count bounds, the cancellable
NeoForge contract, and detached snapshot values.
