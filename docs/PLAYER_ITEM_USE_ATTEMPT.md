# Player item-use attempts

`player.item_use_attempt` is a synchronous, cancellable event from the logical
server. NeoForge emits it after the general `player.interaction` veto check and
before the item receives its `use` call. It applies to right-click item use;
block and entity interactions are handled by `player.interaction`.

Subscribe from a server module and return `Ok(true)` to cancel the item use:

```rust
use sjml::{event, events::{self, EventPriority}, mod_init, SjmlResult};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("player.item_use_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.item_use_attempt")]
fn inspect_use(attempt: sjml::events::PlayerItemUseAttempt) -> SjmlResult<bool> {
    Ok(attempt.item_id == "minecraft:ender_pearl")
}
```

The callback payload contains the canonical player UUID, dimension ID, hand
(`main_hand` or `off_hand`), item registry ID, and count (`0..=1,000,000`). An
empty hand is reported as `minecraft:air` with count zero. Strings are bounded
to 256 UTF-8 bytes. The values are detached snapshots; no Minecraft object or
live item stack crosses the ABI, and the callback cannot replace or modify the
stack. The host runs handlers on the owning logical-server thread and preserves
normal item use when a callback errors or returns an invalid result.

The module and administrator grant both need `events.subscribe` and
`events.cancel`. The event cannot be asynchronous. Its schema-v1 wire layout
and callback export are specified in [ABI.md](ABI.md); general ordering,
priorities, and subscription lifetime are in [GAMEPLAY_EVENTS.md](GAMEPLAY_EVENTS.md).
