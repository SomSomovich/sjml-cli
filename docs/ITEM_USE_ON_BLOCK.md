# Registered item use on blocks

`item.use_on_block` lets a module handle a right-click on a block when the
player holds an item declared by that same module. The callback runs on the
logical server and receives detached values only.

Subscribe from the retained server module during initialization. The callback
is synchronous, requires `events.subscribe` and `events.cancel`, and cannot be
declared with `#[event_async]`.

```rust
use sjml::{
    event,
    events::{self, EventPriority, ItemUseAction, ModuleItemUseOnBlock},
    SjmlResult,
};

pub fn initialize() -> SjmlResult<()> {
    events::subscribe("item.use_on_block", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "item.use_on_block")]
fn handle_item_on_block(use_event: ModuleItemUseOnBlock) -> SjmlResult<ItemUseAction> {
    if use_event.item_id == "example:content/magic_wand"
        && use_event.block_id == "minecraft:stone"
        && use_event.hand == "main_hand"
    {
        Ok(ItemUseAction::Success)
    } else {
        Ok(ItemUseAction::Pass)
    }
}
```

The snapshot contains player UUID, dimension, item ID and count, clicked block
ID and coordinates, face, and hand. It does not expose Minecraft objects. The
host first dispatches `block.use` to the module that owns the clicked block.
If that callback returns `Pass`, `item.use_on_block` is dispatched to the
module that owns the held item. Other modules cannot observe or control that
item through this callback. If both callbacks pass, vanilla block and item
behavior continues.

Return `ItemUseAction::Pass` to continue vanilla behavior, `Success` to handle
the click without consuming an item, or `ConsumeOne` to handle it and remove
one item only when the held item ID and count still match the snapshot after
the callback. A handled action reports NeoForge's successful interaction
result. The host rejects malformed callback results and disables the failing
module callback.

Payloads are limited to 4 KiB, strings to 256 UTF-8 bytes, and stack counts to
1,000,000. The corresponding owner-scoped held-item right-click callback
without a block target is [`item.use`](CUSTOM_ITEM_BEHAVIOR.md); the
cancellable general item veto remains [`player.item_use_attempt`](PLAYER_ITEM_USE_ATTEMPT.md).

The dedicated-server test in
[`ModuleBlockUseGameTests`](../src/main/java/dev/somuch/sjml/neoforge/gametest/ModuleBlockUseGameTests.java)
uses a registered module item on a vanilla stone block and checks successful
handling and single-item consumption.
