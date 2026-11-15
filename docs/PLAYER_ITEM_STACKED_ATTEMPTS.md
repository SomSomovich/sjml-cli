# Player item-stacked attempts

Server modules can synchronously veto a carried item stack stacking onto a
clicked inventory slot through `player.item_stacked_attempt`. NeoForge fires
`ItemStackedOnOtherEvent` before the vanilla `Item#overrideStackedOnOther` and
`Item#overrideOtherStackedOnMe` hooks run, so the callback applies before the
container swaps or merges the carried and slot stacks. The host dispatches the
callback on the logical server's owning thread; already-canceled events,
non-server contexts, players outside a server level, and dedicated or
integrated server mismatches are not dispatched.

The callback requires `events.subscribe` and `events.cancel`. Returning
`Ok(true)` cancels the event with `setCanceled(true)`, which rejects the
stacking interaction and leaves both stacks unchanged; returning `Ok(false)`
keeps the vanilla behavior. `#[event_async]` is rejected because the veto must
apply before NeoForge performs the interaction.

The detached schema-v1 payload contains the player UUID, the dimension ID, and
then two item snapshots in order: `carried` (the stack held by the cursor) and
`stacked_on` (the stack in the clicked slot). Each snapshot carries a nested
schema version, a canonical registry item ID, and a count. After both snapshots
come the lowercase `click_action` and the `slot_index`. An empty stack is
`minecraft:air` with count 0; `minecraft:air` appears exactly when the count is
0. Counts are bounded to `0..=1,000,000`, click actions match `[a-z_]{1,32}`,
slot indexes are bounded to `0..=255`, IDs must be canonical lowercase resource
locations, strings are bounded to 256 UTF-8 bytes, and the payload to 2 KiB. Up
to 64 subscribed handlers run per stacking attempt.

```rust
use sjml::{
    event,
    events::{self, EventPriority},
    player_item_stacked_attempt::PlayerItemStackedAttempt,
    SjmlResult,
};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("player.item_stacked_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.item_stacked_attempt")]
fn veto_pickup_stacking(event: PlayerItemStackedAttempt) -> SjmlResult<bool> {
    Ok(event.click_action == "pickup")
}
```

Only bounded identifiers, item IDs, counts, click actions, and slot indexes
cross the ABI; the event's mutable carried stack, slot contents, and carried
slot access are left untouched, so a callback can neither create nor duplicate
items. Guest errors disable the failing module and leave the vanilla stacking
behavior unchanged.

GameTests make a mock server player, build a real `Slot`, and post a real
`ItemStackedOnOtherEvent` with a diamond carried onto dirt; the `pickup` click
is canceled while a `quick_move` control keeps its vanilla stacking. Unit tests
cover the stable codec, bounds, the cancellable NeoForge contract, and detached
snapshot values.
