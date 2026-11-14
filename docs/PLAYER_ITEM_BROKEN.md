# Player item-broken event

Server-side modules can subscribe to the observational `player.item_broken`
event. NeoForge raises it when a `ServerPlayer` breaks a held item. The
schema-v1 payload contains the player's canonical UUID, dimension ID, broken
item registry ID, and hand (`main`, `off`, or an empty string when NeoForge
does not identify a hand).

```rust
use sjml::events::PlayerItemBroken;
use sjml::{event, SjmlResult};

fn subscribe() -> SjmlResult<()> {
    sjml::events::subscribe("player.item_broken", sjml::events::EventPriority::Normal)
}

#[event(name = "player.item_broken")]
fn item_broken(event: PlayerItemBroken) -> SjmlResult<()> {
    let _ = (event.player_id, event.item_id, event.hand);
    Ok(())
}
```

The module must request `events.subscribe`, and the local server must grant
that capability. The hook runs on the logical-server thread and does not
cancel, replace, or restore the broken item. Payload data is detached from
Minecraft objects; callbacks cannot retain an item stack or player reference.

The codec rejects non-canonical player, dimension, and item IDs and any hand
value other than `main`, `off`, or empty. A guest-driven GameTest breaks a
diamond sword and verifies delivery with the main-hand snapshot.
