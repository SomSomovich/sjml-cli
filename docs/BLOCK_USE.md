# Registered block use

`block.use` lets a registered block handle a logical-server right-click before
Minecraft's vanilla block behavior runs. The host routes the callback only to
the module that declared the clicked block under its `package:module/path`
registry ID. Other modules cannot observe or control that block through this
callback.

Subscribe from the retained server module during initialization. The callback
is synchronous, requires `events.subscribe` and `events.cancel`, and cannot be
declared with `#[event_async]`.

```rust
use sjml::{
    event,
    events::{self, BlockUseAction, EventPriority, ModuleBlockUse},
    SjmlResult,
};

pub fn initialize() -> SjmlResult<()> {
    events::subscribe("block.use", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "block.use")]
fn handle_ruby_lamp_use(use_event: ModuleBlockUse) -> SjmlResult<BlockUseAction> {
    if use_event.face == "up" && use_event.hand == "main_hand" {
        Ok(BlockUseAction::ConsumeOne)
    } else {
        Ok(BlockUseAction::Pass)
    }
}
```

The `ModuleBlockUse` snapshot contains the player UUID, dimension ID, clicked
block ID and coordinates, hit face, hand, held item ID, and stack count. It
contains no live Minecraft objects. The callback runs after the general
`player.interaction` veto, if any.

Return `BlockUseAction::Pass` to continue normal vanilla behavior. Return
`Success` to handle the click without consuming an item. `ConsumeOne` handles
the click and removes one item only when the held item ID and count still match
the snapshot after the callback. A handled action reports NeoForge's successful
interaction result. If no subscribed module owns the clicked block, Minecraft
continues its normal behavior.

The host accepts only one-byte action results `0` (`Pass`), `1` (`Success`),
and `2` (`ConsumeOne`). Invalid results disable the failing module callback.
The guest payload is bounded to 4 KiB; strings are at most 256 UTF-8 bytes and
the held stack count is at most 1,000,000.

The dedicated-server GameTest in
[`ModuleBlockUseGameTests`](../src/main/java/dev/somuch/sjml/neoforge/gametest/ModuleBlockUseGameTests.java)
registers an interactive block, calls the real block-use path, and verifies
that the callback handles the interaction and consumes exactly one diamond.
