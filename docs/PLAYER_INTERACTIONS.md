# Player interaction vetoes

The synchronous `player.interaction` event lets a server-side SJML module
observe and veto a player's right-click on a block, entity, or item use. It is distinct
from `player.interacted`, which remains an observation-only notification for
the broader set of interaction attempts.

The callback receives a detached `PlayerInteractionAttempt` record containing
the player UUID, dimension ID, action, hand, and optional target ID, block
face, and block position. Supported actions are `right_click_block`,
`entity_interact`, `entity_interact_specific`, and `right_click_item`. It never
receives a live Minecraft object. Left-click events are not included. For
`right_click_item`, `target_id` identifies the item held in the active hand;
for block/entity actions it identifies the clicked block or entity type.

Handlers run synchronously on the logical server thread while NeoForge is
processing the original interaction. Return `Ok(true)` to veto it or
`Ok(false)` to let vanilla and other handlers continue. If multiple modules
subscribe, any authorized `true` result vetoes the interaction. A malformed
result or trapped callback disables that module and does not veto by itself.

The module must request both `events.subscribe` and `events.cancel`, and the
administrator must grant both capabilities. The subscription is scoped to
the server module and follows normal event priority ordering.

```rust
use sjml::prelude::*;

#[mod_init]
fn register_interaction_handler() -> SjmlResult<()> {
    events::subscribe("player.interaction", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.interaction")]
fn protect_levers(event: PlayerInteractionAttempt) -> SjmlResult<bool> {
    Ok(event.action == "right_click_block"
        && event.target_id.as_deref() == Some("minecraft:lever"))
}
```

Because this event can cancel Minecraft behavior, it cannot use `#[event_async]`.
Keep handlers bounded and fast; their work runs in the server event path.
