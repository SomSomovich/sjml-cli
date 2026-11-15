# Entity swap-items attempts

Server modules can synchronously veto a main-hand/off-hand exchange through
`entity.swap_items_attempt`. NeoForge fires the hook when a living entity is
about to swap its hands, for example when a player right-clicks with an empty
hand. The host dispatches the callback on the logical server's owning thread;
events already cancelled by another listener, non-server contexts, and entities
outside a server level are not dispatched.

The callback requires `events.subscribe` and `events.cancel`. Returning
`Ok(true)` cancels the exchange and leaves both hands unchanged; returning
`Ok(false)` preserves the vanilla swap. `#[event_async]` is rejected because the
veto must apply before NeoForge exchanges the stacks.

The detached schema-v1 payload contains the entity UUID, its registry type, and
the dimension ID, followed by two item snapshots in swap order: `to_main_hand`
(the stack currently in the off hand) and `to_off_hand` (the stack currently in
the main hand). Each snapshot carries a canonical registry item ID and a count.
An empty stack is `minecraft:air` with count 0; `minecraft:air` appears exactly
when the count is 0. Counts are bounded to `0..=1,000,000`, IDs must be
canonical lowercase resource locations, strings are bounded to 256 UTF-8 bytes,
and the payload to 2 KiB. Up to 64 subscribed handlers run per swap.

```rust
use sjml::{
    entity_swap_items_attempt::EntitySwapItemsAttempt,
    event,
    events::{self, EventPriority},
    SjmlResult,
};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("entity.swap_items_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.swap_items_attempt")]
fn veto_zombie_hand_swap(event: EntitySwapItemsAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:zombie")
}
```

Only bounded identifiers, item IDs, and counts cross the ABI; the event's
mutable stacks are left untouched, so a callback can neither create nor
duplicate items. Guest errors disable the failing module and leave the swap
unchanged.

GameTests spawn a zombie and a cow with an iron sword in the main hand and a
shield in the off hand, post a real `LivingSwapItemsEvent.Hands`, and verify
that only the zombie swap is canceled. Unit tests cover the stable codec,
bounds, the cancellable NeoForge contract, and detached snapshot values.
