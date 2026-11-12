# Cancellable block-break attempts

`block.break_attempt` is a synchronous server event for a player block-break attempt. A guest can reject the operation before Minecraft removes the block.

## NeoForge hook and ordering

SJML adapts NeoForge 21.1.265 `BlockEvent.BreakEvent`. Minecraft posts this event from the server-side block-destruction path before `playerWillDestroy`, block removal, and loot handling. Cancelling it makes the server report that the break failed and sends the current block state back to the client.

The callback runs inline on the owning server thread. It receives a detached record, never a Minecraft `Player`, `Level`, `BlockState`, or `BlockPos` object. This means the callback must finish quickly and must not wait for another server task.

SJML calls `block.break_attempt` before the older `block.break` callback. The existing `block.break` subscription remains available with its existing payload and cancellation behavior. If another listener has already cancelled the NeoForge event before SJML receives it, SJML skips `block.break_attempt`; the older callback keeps its existing delivery behavior. Any `true` result from either SJML event cancels the operation.

## SDK callback

```rust
use sjml::prelude::*;

#[event(name = "block.break_attempt")]
fn protect_spawn(event: BlockBreakAttempt) -> SjmlResult<bool> {
    let protected = event.dimension_id == "minecraft:overworld"
        && event.x.abs() <= 8
        && event.z.abs() <= 8;
    Ok(protected)
}
```

Return `Ok(true)` to veto; return `Ok(false)` to allow this handler's vote. All subscribed handlers are called in priority order, and any authorized `true` veto wins. A module failure does not implicitly veto; the failing module is disabled and other handlers still run.

## Payload schema v1

The StableEncode record is `BlockBreakAttempt`, in this field order:

1. `player_id`: canonical UUID string of the player.
2. `dimension_id`: dimension resource location.
3. `block_id`: registry ID of the block type at the position; state properties are omitted.
4. `x`, `y`, `z`: signed block coordinates.

The record includes no mutable world objects. Each string is bounded to 65,536 UTF-8 bytes and the full ABI payload is bounded to 1 MiB.

## Permissions and scope

The module must declare and receive both `events.subscribe` and `events.cancel`. The first permits event subscriptions; the second authorizes a veto. The adapter dispatches only for a `ServerLevel`, on its owning server thread, and only to modules active in that server execution context. Client-only module instances never receive this event.

Use this callback for quick policy checks. Expensive work belongs outside the break path; asynchronous callbacks cannot veto an operation after it has been accepted.

## GameTest coverage

`BlockBreakAttemptGameTests` drives `ServerPlayerGameMode.destroyBlock` against the real NeoForge hook. One test cancels `BlockEvent.BreakEvent` and verifies the block remains; another leaves the event uncancelled and verifies normal removal. Host dispatcher tests separately exercise authorized guest boolean callbacks and server-context enforcement.
