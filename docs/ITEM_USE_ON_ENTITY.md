# Registered item use on entities

`item.use_on_entity` lets a module handle a right-click on an entity while a
module-owned item is held. The callback receives detached player, item, and
target-entity values; it never exposes a live Minecraft object.

Subscribe from the retained server module during initialization. The callback
is synchronous, requires `events.subscribe` and `events.cancel`, and cannot be
declared with `#[event_async]`.

```rust
use sjml::{
    event,
    events::{self, EventPriority, ItemUseAction, ModuleItemUseOnEntity},
    SjmlResult,
};

pub fn initialize() -> SjmlResult<()> {
    events::subscribe("item.use_on_entity", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "item.use_on_entity")]
fn use_wand_on_entity(attempt: ModuleItemUseOnEntity) -> SjmlResult<ItemUseAction> {
    if attempt.item_id == "example:content/healing_wand"
        && attempt.target_type_id == "minecraft:cow"
    {
        Ok(ItemUseAction::Success)
    } else {
        Ok(ItemUseAction::Pass)
    }
}
```

NeoForge first gives the more specific entity `interactAt` stage an opportunity
to succeed. If it passes, the general `EntityInteract` event runs. SJML applies
the general `player.interaction` veto first, then dispatches `item.use_on_entity`
to only the module that declared the held item, before vanilla entity and item
interaction behavior. This lets item callbacks avoid overriding entity-specific
parts and normal player interaction cancellation still takes precedence. If
the module callback returns `Pass`, vanilla behavior continues.

The snapshot includes the player's UUID and dimension, held item ID and count,
target entity UUID and type ID, target coordinates, and hand. Coordinates must
be finite and within ±30,000,000; strings are capped at 256 UTF-8 bytes, stack
counts at 1,000,000, and the payload at 4 KiB.

Return `Pass` to continue vanilla behavior, `Success` to handle the click
without consuming an item, or `ConsumeOne` to handle it and consume one item
only if its ID and count remain unchanged after the callback. Invalid callback
results disable the failing module callback. The result uses NeoForge's
successful cancellation result.

The dedicated-server GameTest in
[`ModuleBlockUseGameTests`](../src/main/java/dev/somuch/sjml/neoforge/gametest/ModuleBlockUseGameTests.java)
posts the general entity interaction to NeoForge's event bus, then verifies
owner routing, successful handling, and single-item consumption against a live
cow and registered item.
