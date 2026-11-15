# Piston movement attempts

Server modules can veto a piston move before any block state changes:

```rust
use sjml::{event, events::{self, BlockPistonAttempt, EventPriority}, SjmlResult};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("block.piston_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "block.piston_attempt")]
fn stop_eastward_pistons(event: BlockPistonAttempt) -> SjmlResult<bool> {
    Ok(event.extending && event.direction == "east")
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's cancellable `PistonEvent.Pre`. Returning `Ok(true)` prevents the
movement entirely: no moved blocks, no block updates, and no
`PistonEvent.Post`. Returning `Ok(false)` keeps vanilla behavior.

The detached schema-v1 payload contains the dimension ID, piston block x/y/z,
the lowercase facing name, an `extending` flag (true for extension, false for
retraction), the canonical piston block ID, and the piston's canonical block
state text (at most 512 bytes). The position is bounded to ±30,000,000 in x/z;
strings are bounded to 256 UTF-8 bytes and the payload to 2 KiB. Up to 64
subscribed handlers run per attempt, and all subscriptions require both
`events.subscribe` and `events.cancel`. `#[event_async]` is rejected because the
veto must apply before the piston updates block states.

GameTests post a real `PistonEvent.Pre` for an eastward extension and a
westward control and verify that only the marked movement is canceled. Unit
tests cover the stable codec, bounds, the cancellable NeoForge contract, and
detached snapshot values.
