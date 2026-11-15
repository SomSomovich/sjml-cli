# Player sweep attack attempts

Server modules can veto a player's sweep attack before any sweep damage occurs:

```rust
use sjml::{
    event,
    events::{self, EventPriority},
    player_sweep_attack_attempt::PlayerSweepAttackAttempt,
    SjmlResult,
};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("player.sweep_attack_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.sweep_attack_attempt")]
fn stop_cow_sweeps(event: PlayerSweepAttackAttempt) -> SjmlResult<bool> {
    Ok(event.target_type_id == "minecraft:cow" && event.vanilla_sweep)
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's cancellable `SweepAttackEvent`. The event fires on both logical
sides, but the host dispatches it only for logical-server contexts on the
server thread that owns the player's level. Returning `Ok(true)` cancels the
event and sets `sweeping` to false, so the attack deals no sweep damage.
Returning `Ok(false)` keeps vanilla behavior. Already-canceled events are
ignored.

The detached schema-v1 payload contains, in order, the player's canonical
UUID, the dimension resource location, the target entity's canonical UUID, the
target entity type registry ID, and a `vanilla_sweep` flag reporting whether
vanilla's own sweep rules would trigger a sweep. Each string is bounded to 256
UTF-8 bytes and the payload to 2 KiB. Up to 64 subscribed handlers run per
attempt, and all subscriptions require both `events.subscribe` and
`events.cancel`. `#[event_async]` is rejected because the veto must apply
before the attack applies its sweep result.

GameTests post a real `SweepAttackEvent` for a cow with `vanilla_sweep` true
and a pig control with `vanilla_sweep` false, then verify that only the marked
sweep is canceled and has `sweeping` disabled. Unit tests cover the stable
codec, bounds, the cancellable NeoForge contract, and detached snapshot values.
