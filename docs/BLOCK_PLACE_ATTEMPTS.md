# Cancellable block-placement attempts

`block.place_attempt` is a synchronous logical-server event fired while
NeoForge is processing a block placement. A handler can veto the placement
before the successful-placement notification `block.place` is delivered. The
existing `block.place` event remains a non-cancellable notification for a
placement that proceeds.

The detached `BlockPlaceAttempt` payload carries an optional placer UUID, the
dimension ID, proposed block ID, and event block coordinates. It does not
expose a live player, block, or world object. NeoForge's
`BlockEvent.EntityPlaceEvent` is cancellable; when an authorized handler vetoes
the event, NeoForge cancels and restores the placement snapshots. For a
multi-place event, SJML delivers one payload for NeoForge's event position, and
a veto rejects the complete placement operation.

Handlers run synchronously on the server thread. Return `Ok(true)` to veto or
`Ok(false)` to continue. When multiple modules subscribe, any authorized
`true` result vetoes the placement. A malformed result or failing callback
disables that module and does not veto by itself.

The module must request both `events.subscribe` and `events.cancel`, and the
administrator must grant both capabilities.

```rust
use sjml::prelude::*;

#[mod_init]
fn register_placement_guard() -> SjmlResult<()> {
    events::subscribe("block.place_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "block.place_attempt")]
fn deny_diamond_blocks(event: BlockPlaceAttempt) -> SjmlResult<bool> {
    Ok(event.block_id == "minecraft:diamond_block")
}
```

Because this callback can cancel game behavior, it cannot use `#[event_async]`.
Keep handlers bounded and fast; their work runs inside NeoForge's placement
event path.
