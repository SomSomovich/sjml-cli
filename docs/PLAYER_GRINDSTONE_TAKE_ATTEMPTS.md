# Grindstone take attempts

Server modules can veto a grindstone output take before vanilla consumes the
inputs:

```rust
use sjml::{
    event,
    events::{self, EventPriority},
    player_grindstone_take_attempt::PlayerGrindstoneTakeAttempt,
    SjmlResult,
};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("player.grindstone_take_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.grindstone_take_attempt")]
fn block_diamond_sword_takes(event: PlayerGrindstoneTakeAttempt) -> SjmlResult<bool> {
    Ok(event.top.item_id == "minecraft:diamond_sword")
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's cancellable `GrindstoneEvent.OnTakeItem`, which fires when a player
takes the grindstone output. Returning `Ok(true)` cancels the take: the output
stays in place, neither input is consumed, and no experience is awarded.
Returning `Ok(false)` keeps vanilla behavior.

The veto is all-or-nothing: there is no per-slot cancellation, and the host does
not write back item stacks, so a callback can neither create, remove, nor
duplicate items. The host listener skips canceled events, non-server contexts,
players outside a `ServerLevel`, and dedicated/integrated mismatches, and
rejects captures made off the owning logical-server thread.

The detached schema-v1 payload contains the player UUID, the dimension ID, and
four item snapshots in the order top, bottom, new top, and new bottom. Each
nested item snapshot carries its own `i32` schema, a canonical item ID, and a
count. An empty stack is encoded as `minecraft:air` with count 0; only
`minecraft:air` may use count 0, any other stack uses a count of 1..=1,000,000.
The experience amount is bounded to `0..=1,000`. The little-endian wire layout
is `i32 schema=1`, the player UUID string, the dimension ID string, then the
four nested item snapshots, and finally an `i32` experience amount. Strings are
bounded to 256 UTF-8 bytes and the payload to 4 KiB. Up to 64 subscribed
handlers run per attempt, and all subscriptions require both `events.subscribe`
and `events.cancel`. `#[event_async]` is rejected because the veto must be
applied before the output is taken.

GameTests post a real `GrindstoneEvent.OnTakeItem` with a diamond sword and an
iron sword top input and verify that only the marked take is canceled. Unit
tests cover both payload variants, the empty-stack identity rule, experience
bounds, the cancellable NeoForge contract, and detached snapshot values.
