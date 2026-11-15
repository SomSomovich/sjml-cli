# Player critical-hit attempts

Server modules can disable the critical-hit component of a player attack before
damage is applied:

```rust
use sjml::{
    event,
    events::{self, EventPriority},
    player_critical_hit_attempt::PlayerCriticalHitAttempt,
    SjmlResult,
};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("player.critical_hit_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.critical_hit_attempt")]
fn disable_cow_critical_hits(event: PlayerCriticalHitAttempt) -> SjmlResult<bool> {
    Ok(event.target_type_id == "minecraft:cow" && event.vanilla_critical)
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's `CriticalHitEvent`. It fires on both logical sides, but modules only
receive it for logical-server contexts on the owning server thread. The hook is
not cancellable: returning `Ok(true)` calls `event.setCriticalHit(false)`, which
disables the critical hit while the attack still lands. Returning `Ok(false)`
leaves both the critical flag and the multiplier unchanged.

The detached schema-v1 payload contains the player UUID, the dimension ID, the
target entity UUID and type ID, the vanilla critical flag, the vanilla
multiplier, and the current damage multiplier. Strings are bounded to 256 UTF-8
bytes, both floats must be finite and within 0.0 to 1,000.0, and the payload is
capped at 2 KiB. Up to 64 subscribed handlers run per attempt, and all
subscriptions require both `events.subscribe` and `events.cancel`.
`#[event_async]` is rejected because the veto must apply before the attack
damage is computed.

GameTests post a real `CriticalHitEvent` for a cow and a pig control and verify
that only the marked cow critical hit is disabled. Unit tests cover the stable
codec, float bounds, the non-cancellable NeoForge contract, and detached
snapshot values.
