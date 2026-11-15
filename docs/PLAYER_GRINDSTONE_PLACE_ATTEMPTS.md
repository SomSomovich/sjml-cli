# Grindstone place attempts

Server modules can veto a grindstone input placement before vanilla computes the
output:

```rust
use sjml::{
    event,
    events::{self, EventPriority},
    player_grindstone_place_attempt::PlayerGrindstonePlaceAttempt,
    SjmlResult,
};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("player.grindstone_place_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.grindstone_place_attempt")]
fn block_diamond_sword_placements(event: PlayerGrindstonePlaceAttempt) -> SjmlResult<bool> {
    Ok(event.top.item_id == "minecraft:diamond_sword")
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's cancellable `GrindstoneEvent.OnPlaceItem`, which fires when the
grindstone inputs change. Returning `Ok(true)` cancels the placement: vanilla
does not compute an output and the output slot becomes empty. Returning
`Ok(false)` keeps vanilla behavior.

The veto is all-or-nothing: there is no per-slot cancellation, and the host does
not write back item stacks, so a callback can neither create, remove, nor
duplicate items. The event carries no level or player, so the adapter itself
does not check the server thread; the host listener performs the current-server
and owning-thread checks before dispatching, and skips canceled events and
non-server contexts.

The detached schema-v1 payload contains two item snapshots in the order top and
bottom, followed by the experience amount. Each nested item snapshot carries its
own `i32` schema, a canonical item ID, and a count. An empty stack is encoded as
`minecraft:air` with count 0; only `minecraft:air` may use count 0, any other
stack uses a count of 1..=1,000,000. The experience amount is bounded to
`0..=1,000`. The little-endian wire layout is `i32 schema=1`, the top item
snapshot, the bottom item snapshot, and finally an `i32` experience amount.
Strings are bounded to 256 UTF-8 bytes and the payload to 2 KiB. Up to 64
subscribed handlers run per attempt, and all subscriptions require both
`events.subscribe` and `events.cancel`. `#[event_async]` is rejected because the
veto must be applied before the output is computed.

GameTests post a real `GrindstoneEvent.OnPlaceItem` with a diamond sword and an
iron sword top input and verify that only the marked placement is canceled.
Unit tests cover both payload variants, the empty-slot identity rule,
experience bounds, the cancellable NeoForge contract, and detached snapshot
values.
