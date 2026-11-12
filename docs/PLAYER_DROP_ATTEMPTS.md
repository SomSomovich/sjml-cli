# Cancellable player item drops

Server modules can receive `player.drop_attempt` for an item stack a player tosses from an inventory. The callback runs synchronously on the logical server, before NeoForge adds the tossed item entity to the world. Its detached payload contains the player UUID, dimension ID, item ID, stack count, and a `can_cancel` flag.

Subscribe with `events.subscribe`. Veto handlers also need `events.cancel` and return `Ok(true)` when they want to prevent the toss:

```rust
use sjml::{event, PlayerDropAttempt, SjmlResult};

#[event(name = "player.drop_attempt")]
fn keep_rare_drops(event: PlayerDropAttempt) -> SjmlResult<bool> {
    Ok(event.item_id == "example:protected_token" && event.can_cancel)
}
```

`can_cancel` is true only when the entire stack can fit back into the player's inventory. NeoForge fires its toss event after the stack has left its source slot, so the host restores the stack before canceling. If the inventory is full, the callback still observes the attempt, but a `true` result cannot cancel it; the ordinary drop continues. If a callback changes the inventory so restoration becomes incomplete, the host leaves only the un-restored remainder in the world. This avoids deleting or duplicating items.

The event reports stack counts from 1 to 99. Unknown items, oversized or malformed payloads, client-side tosses, and events already canceled by another handler are skipped. It does not expose live Minecraft objects.
