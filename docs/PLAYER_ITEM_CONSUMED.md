# Player completed item-use event

Logical-server modules can observe a server player completing an item use with
`player.item_consumed`. NeoForge's `LivingEntityUseItemEvent.Finish` fires after
the item has applied its use effects. The callback is observational: this hook
does not implement `ICancellableEvent`, and the event cannot undo effects or
change the replacement stack.

```rust
use sjml::events::PlayerItemConsumed;
use sjml::{event, SjmlResult};

fn subscribe() -> SjmlResult<()> {
    sjml::events::subscribe("player.item_consumed", sjml::events::EventPriority::Normal)
}

#[event(name = "player.item_consumed")]
fn item_consumed(event: PlayerItemConsumed) -> SjmlResult<()> {
    let _ = (
        event.player_id,
        event.dimension_id,
        event.hand,
        event.item_id,
        event.count,
        event.result_item_id,
        event.result_count,
    );
    Ok(())
}
```

The schema-v1 snapshot contains the canonical player UUID, dimension ID, hand,
input item registry ID and its stack count before finishing, and the result item
registry ID and stack count. An empty result is represented by an empty result
ID and count zero. This records a completed use; it does not assert that the
input item was removed from the inventory. For example, bows and other held
items can finish a use without being consumed. Stack components and live player
or item objects are never exposed. Strings are limited to 256 UTF-8 bytes and
both stack counts are bounded by 1,000,000.

The module needs the `events.subscribe` capability. The callback runs on the
owning logical-server thread, only for `ServerPlayer` instances, and is delivered
to server-side modules. The GameTest fixture posts NeoForge's actual `Finish`
event on the server event bus and checks that the Rust callback receives the
player, hand, pre-use stack, and result stack snapshot.
