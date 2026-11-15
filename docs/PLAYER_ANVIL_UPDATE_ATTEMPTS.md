# Anvil result attempts

Server modules can veto an anvil result before the menu exposes it:

```rust
use sjml::{event, events::{self, EventPriority, PlayerAnvilUpdateAttempt}, SjmlResult};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("player.anvil_update_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.anvil_update_attempt")]
fn block_marked_renames(event: PlayerAnvilUpdateAttempt) -> SjmlResult<bool> {
    Ok(event.name.as_deref() == Some("forbidden"))
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's cancellable `AnvilUpdateEvent`, which fires whenever an anvil input
slot or the name changes. Returning `Ok(true)` cancels the result: vanilla logic
is skipped and the output slot becomes empty. Returning `Ok(false)` leaves the
vanilla result unchanged. The callback cannot set a custom output or cost.

The detached schema-v1 payload contains the player UUID, the left and right
input item IDs and counts (empty stacks use `minecraft:air` with count zero), an
optional client-supplied name (1–128 UTF-8 bytes), and the current level cost
(`0..=1,000,000,000`). IDs must be canonical lowercase resource locations; item
counts are bounded to `0..=1,000,000`; the payload is capped at 2 KiB. Up to 64
subscribed handlers run per attempt, and all subscriptions require both
`events.subscribe` and `events.cancel`. `#[event_async]` is rejected because the
veto must be applied before the menu reads the result.

GameTests post a real `AnvilUpdateEvent` with a marked and an unmarked name and
verify that only the marked result is canceled. Unit tests cover both payload
variants, bounds, the cancellable NeoForge contract, and detached snapshot
values.
