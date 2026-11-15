# Player continue sleeping attempts

Server modules can force a sleeping player to wake through the synchronous
logical-server event `player.continue_sleeping_attempt`:

```rust
use sjml::{
    event,
    events::{self, EventPriority},
    player_continue_sleeping_attempt::PlayerContinueSleepingAttempt,
    SjmlResult,
};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("player.continue_sleeping_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.continue_sleeping_attempt")]
fn wake_sleepers_without_a_problem(event: PlayerContinueSleepingAttempt) -> SjmlResult<bool> {
    Ok(event.problem.is_empty())
}
```

NeoForge 21.1.265 fires `CanContinueSleepingEvent` while checking whether a
sleeping entity may keep sleeping. It extends `LivingEvent` and is not an
`ICancellableEvent`; it exposes `getProblem`, `mayContinueSleeping`, and
`setContinueSleeping`. Returning `Ok(true)` calls `setContinueSleeping(false)`,
so the sleeper is forced to wake. Returning `Ok(false)` preserves the result
computed before the callback. The event is fired only on the logical server.

The detached schema-v1 payload contains the sleeper's canonical UUID, the
dimension ID, the current `problem`, and `may_continue_sleeping`. `problem` is
the lowercase `BedSleepingProblem` name, or an empty string when the event was
fired without a problem. The adapter ignores non-server contexts, entities
outside a server level, and dedicated/integrated context mismatches, and
capturing off the owning server thread throws `IllegalStateException`.

Strings are bounded to 256 UTF-8 bytes, the payload is capped at 1 KiB, and up
to 64 subscribed handlers run per attempt. All subscriptions require both
`events.subscribe` and `events.cancel`. `#[event_async]` is rejected because the
veto must apply before vanilla returns from the sleep check.

GameTests post a real `CanContinueSleepingEvent` without a problem and a control
with `NOT_POSSIBLE_NOW`, verifying that the guest forces only the allowed check
to deny continued sleeping. Unit tests cover the stable codec, string and
payload bounds, the non-cancellable NeoForge contract, and detached snapshot
values.
