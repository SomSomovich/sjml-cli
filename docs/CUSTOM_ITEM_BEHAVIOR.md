# Module-owned item use behavior

An item registered by a module can handle its own right-click use through the
synchronous `item.use` event. The host routes the callback only to the module
whose declared item ID matches `package_id:module_id/path`. Other modules do
not receive the item-use snapshot or control the result.

Subscribe from the server module and return one of the bounded outcomes:

```rust
use sjml::{
    event,
    events::{self, EventPriority, ItemUseAction, ModuleItemUse},
    mod_init, SjmlResult,
};

#[mod_init]
fn register_use_behavior() -> SjmlResult<()> {
    events::subscribe("item.use", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "item.use")]
fn use_ruby(attempt: ModuleItemUse) -> SjmlResult<ItemUseAction> {
    if attempt.item_id == "example:content/ruby" {
        Ok(ItemUseAction::ConsumeOne)
    } else {
        Ok(ItemUseAction::Pass)
    }
}
```

`Pass` lets Minecraft's item implementation run. `Success` handles the click
without changing the stack. `ConsumeOne` handles the click and removes one item
only if the held stack still has the same item ID and count captured before the
callback. A handled result sets NeoForge's cancellation result to success. The
callback can call other SJML server APIs to apply effects; each such operation
still requires its own declared and administrator-granted capability.

Registered items can also declare a vanilla use animation and bounded hold
duration through [`ITEM_USE_BEHAVIOR.md`](ITEM_USE_BEHAVIOR.md). This changes
the registered item's `getUseAnimation` and `getUseDuration` values; it does not
implement custom item effects or change vanilla item behavior.

The callback runs on the logical server after the broad
`player.item_use_attempt` cancellation hook, and only when vanilla has reached
the right-click-item phase. Using an item on a block or entity continues through
the separate `player.interaction` hook and the owner-scoped
[`item.use_on_block`](ITEM_USE_ON_BLOCK.md) or
[`item.use_on_entity`](ITEM_USE_ON_ENTITY.md) callbacks. The payload contains the player UUID,
dimension, hand, item ID, and bounded stack count. It never exposes a Java
object, item stack, arbitrary component, or client-controlled state. The action
is synchronous and server-authoritative.

The module must declare and receive `events.subscribe` and `events.cancel`.
These grants permit the callback subscription and its ability to handle or
consume an item. See [PLAYER_ITEM_USE_ATTEMPT.md](PLAYER_ITEM_USE_ATTEMPT.md)
for the broader veto hook and [ABI.md](ABI.md) for event wire details.
