# Item expiry attempts

Server modules can keep a dropped item from despawning when its age reaches the
vanilla lifespan:

```rust
use sjml::{event, events::{self, EventPriority}, ItemExpireAttempt, SjmlResult};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("item.expire_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "item.expire_attempt")]
fn keep_diamonds_dropped(event: ItemExpireAttempt) -> SjmlResult<bool> {
    Ok(event.item_id == "minecraft:diamond")
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's `ItemExpireEvent`, which is fired once an `ItemEntity` has reached
its maximum lifespan. Returning `Ok(true)` makes the host call
`event.setExtraLife(Short.MAX_VALUE)`, which clears the expiry for this attempt
and leaves the dropped item in the world for another 32,767 ticks (about 27.5
minutes). Returning `Ok(false)` leaves the extra life at zero, so vanilla
removes the item as usual. `ItemExpireEvent` is not cancellable, so the host
maps the guest verdict to extra life instead of canceling the event.

The detached schema-v1 payload contains the canonical item-entity UUID, the
registry ID of the dropped item, the stack count (1 to 1,000,000), the
dimension ID, the entity position as little-endian `f64` x/y/z values, and the
current entity age in ticks (0 to 1,000,000). Coordinates must be finite and
within ±30,000,000; strings are bounded to 256 UTF-8 bytes and the payload to
1 KiB. Up to 64 subscribed handlers run per attempt, and all subscriptions
require both `events.subscribe` and `events.cancel`. `#[event_async]` is
rejected because the extra life must be set before the expiry check reads it.

The host ignores events from non-server contexts, item entities that are not in
a `ServerLevel`, and stacks whose item is not registered; dedicated and
integrated server contexts must match. Unit tests cover the stable codec,
count/age/coordinate and ID bounds, the NeoForge extra-life contract, and
detached snapshot values. GameTests spawn a real diamond item and a dirt item
control, post `ItemExpireEvent` for both, and verify that the guest extends only
the diamond.
