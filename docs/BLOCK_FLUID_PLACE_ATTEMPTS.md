# Fluid block placement attempts

A module can subscribe to NeoForge's logical-server `BlockEvent.FluidPlaceBlockEvent`
through the synchronous cancellable `block.fluid_place_attempt` event. This hook
runs when a fluid is about to place or replace a block, including fluid
interactions that turn a target into another block. A guest can veto that one
block-state placement; it cannot stop the fluid from flowing or change later
fluid ticks.

```rust
use sjml::{events, BlockFluidPlaceAttempt, EventPriority, SjmlResult};

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("block.fluid_place_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "block.fluid_place_attempt")]
fn prevent_lava_fire(attempt: BlockFluidPlaceAttempt) -> SjmlResult<bool> {
    Ok(attempt.fluid_id == "minecraft:lava" && attempt.result_block_id == "minecraft:fire")
}
```

The payload contains the dimension, target position, source-liquid position,
fluid ID, original block ID, and intended result block ID. Positions and
resource locations are detached values; block-state properties and Minecraft
objects never cross the ABI. Strings are canonical lowercase resource
locations limited to 256 bytes each, and the entire payload is limited to 2
KiB. The source fluid is read only from the position NeoForge supplied, which
is already in the active level; this API does not load chunks.

The callback runs synchronously on the owning logical-server thread. Returning
`true` cancels the event; `false` leaves the existing decision unchanged. A
callback error fails open for this fluid placement and disables only its
module. Both `events.subscribe` and `events.cancel` are required, and async
handlers are rejected because the fluid hook needs the result before Minecraft
continues.

The integration test posts NeoForge's actual cancellable event type through the
live event bus and verifies that the guest receives the bounded snapshot and
that the target block stays unchanged after a veto. It does not attempt to
cover every vanilla fluid interaction or third-party fluid implementation.
