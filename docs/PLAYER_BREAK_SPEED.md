# Player block-break speed

`player.break_speed` lets a logical-server module preserve, replace, or cancel
the speed NeoForge is about to use for a player's block-breaking calculation.
It is backed by `PlayerEvent.BreakSpeed`, whose supported API exposes the
original and current speed, a speed setter, and cancellation. SJML dispatches
only for a live `ServerPlayer` on its owning server thread; client-side event
calls are ignored. The server remains authoritative for block-breaking
progress, so local client prediction may briefly use vanilla speed.

## Subscription and callback

Subscribe during module initialization. The synchronous callback receives only
detached player, dimension, block, optional position, and speed values:

```rust
use sjml::{
    event,
    events::{self, EventPriority, PlayerBreakSpeed, PlayerBreakSpeedAction},
    mod_init, SjmlResult,
};

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("player.break_speed", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.break_speed")]
fn adjust_speed(event: PlayerBreakSpeed) -> SjmlResult<PlayerBreakSpeedAction> {
    if event.block_id == "minecraft:stone" {
        Ok(PlayerBreakSpeedAction::SetSpeed(12.0))
    } else {
        Ok(PlayerBreakSpeedAction::Pass)
    }
}
```

The module must request and receive both `events.subscribe` and
`events.cancel` to subscribe. Returning `SetSpeed` additionally requires the
`player.break_speed.modify` capability. `Cancel` uses NeoForge's cancellable
event mechanism and prevents this break attempt. An unapproved `SetSpeed` is
ignored.

Handlers run synchronously on the owning server thread; `#[event_async]` is not
supported. At most 64 subscribed callbacks run for one speed calculation.
Handlers are ordered by event priority and then registration order. `Pass`
leaves the current value unchanged and continues. An authorized `SetSpeed`
replaces the current value and later handlers see that updated value. A
`Cancel` stops dispatch and rejects the attempt. Invalid action bytes or a
callback failure disable that module and leave the current speed available to
remaining handlers.

## Bounds and payload

The schema-v1 payload contains the canonical player UUID, dimension and block
IDs, an optional block position, the original speed, and the speed currently
proposed by prior event listeners. Speed snapshots must be finite and within
`-1.0..=1024.0`; `-1.0` is preserved for a block NeoForge considers
unbreakable. Replacement speeds must be finite and within `0.0..=1024.0`.
NeoForge may omit the position for legacy calls, which the SDK represents as
`None`.

The action response is unversioned: byte `0` means `Pass`, byte `1` means
`Cancel`, and byte `2` followed by a little-endian `f32` means `SetSpeed`.
The host validates response length, finite values, and bounds before changing
the event.
