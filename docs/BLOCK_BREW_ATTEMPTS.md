# Brewing attempts

Server modules can veto a brewing-stand operation before vanilla consumes the
ingredient:

```rust
use sjml::{
    block_brew_attempt::BlockBrewAttempt,
    event,
    events::{self, EventPriority},
    SjmlResult,
};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("block.brew_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "block.brew_attempt")]
fn stop_awkward_brews(event: BlockBrewAttempt) -> SjmlResult<bool> {
    Ok(event
        .stacks
        .first()
        .is_some_and(|stack| stack.item_id == "minecraft:potion")
        && event
            .stacks
            .last()
            .is_some_and(|stack| stack.item_id == "minecraft:nether_wart"))
}
```

The event runs synchronously through NeoForge's cancellable
`PotionBrewEvent.Pre`, which fires before vanilla brewing consumes the
ingredient. Returning `Ok(true)` cancels the entire operation: the three bottle
slots and the ingredient keep their contents and no `PotionBrewEvent.Post`
runs. Returning `Ok(false)` keeps vanilla behavior.

The veto is all-or-nothing: there is no per-slot cancellation, and the host does
not write back item stacks, so a callback can neither create, remove, nor
duplicate items. `PotionBrewEvent.Pre` carries no level or position; the host
listener performs the current-server and owning-thread checks before invoking
handlers, so the payload carries only item identities.

The detached schema-v1 payload contains up to four slots in brewing-stand order
(three bottles followed by the ingredient). Each slot is a canonical item ID and
a count; an empty slot is encoded as `minecraft:air` with count 0. Only
`minecraft:air` may use count 0, any other stack uses a count of
1..=1,000,000, and the list itself may be empty. The little-endian wire layout
is `i32 schema=1`, `u32 stack_count`, then per slot a nested `i32` snapshot
schema, the item ID string, and a `u32` count. Strings are bounded to 256 UTF-8
bytes and the payload to 2 KiB. Up to 64 subscribed handlers run per attempt,
and all subscriptions require both `events.subscribe` and `events.cancel`.
`#[event_async]` is rejected because the veto must apply before brewing consumes
the ingredient.

GameTests post a real `PotionBrewEvent.Pre` for a potion and nether wart mix and
for a sugar control, and verify that only the marked mix is canceled. Unit tests
cover the stable codec, bounds, the cancellable NeoForge contract, and detached
snapshot values.
