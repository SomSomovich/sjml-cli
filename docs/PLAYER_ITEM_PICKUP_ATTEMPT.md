# Cancellable item pickup attempts

`player.item_pickup_attempt` lets a logical-server module veto an item entity
pickup before Minecraft inserts any of its contents into a player's inventory.
Subscribe in `mod_init`; return `Ok(true)` to leave the item in the world:

```rust
use sjml::{event, events::{self, EventPriority, PlayerItemPickupAttempt}, mod_init, SjmlResult};

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("player.item_pickup_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.item_pickup_attempt")]
fn protect_special_items(event: PlayerItemPickupAttempt) -> SjmlResult<bool> {
    Ok(event.item_id == "example:protected_token")
}
```

The module must request and receive both `events.subscribe` and `events.cancel`.
The callback is synchronous because its boolean sets NeoForge's pickup decision;
it cannot use `#[event_async]`. A true result uses
`ItemEntityPickupEvent.Pre.setCanPickup(TriState.FALSE)` before the inventory is
touched. A false result leaves the normal pickup rules and other event handlers
in control. If another listener has already denied pickup, SJML does not dispatch
the callback or change that result. The event cannot change the stack, inventory,
pickup delay, or pickup target.

NeoForge's Pre event is not marked cancellable: a veto is expressed by setting its
`TriState` to `FALSE`. SJML listens at `LOWEST`; another listener registered later
at the same priority can still explicitly replace that value.

The host listens at `LOWEST` on the owning logical-server thread. Physical-client
contexts and items in a different level from the player are ignored. The schema-v1
snapshot contains only the canonical player UUID, item ID, item count,
and item-entity X/Y/Z. It does not include item components or NBT. Counts are
bounded to `1..=1_000_000`; IDs to 256 UTF-8 bytes; coordinates must be finite and
within ±30,000,000; and the payload is limited to 2 KiB. Invalid or oversized
snapshots are skipped without changing vanilla pickup behavior.

`PlayerItemPickupAttemptGameTests` invokes `ItemEntity.playerTouch` against a
real test server. It verifies an allowed stick follows vanilla pickup and a
vetoed diamond stack remains intact in the world without entering inventory.
