# Cancellable block growth attempts

`block.grow_attempt` lets a logical-server module veto NeoForge's pre-growth event for crops and
other growing-age blocks. Return `Ok(true)` to prevent the pending growth:

```rust
use sjml::{event, events::{self, BlockGrowAttempt, EventPriority}, mod_init, SjmlResult};

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("block.grow_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "block.grow_attempt")]
fn protect_crops(event: BlockGrowAttempt) -> SjmlResult<bool> {
    Ok(event.block_id == "minecraft:wheat" && event.position.y > 100)
}
```

The module must request and receive both `events.subscribe` and `events.cancel`. The host listens
to `CropGrowEvent.Pre`, which NeoForge fires only on the logical server during random-tick growth
attempts. A `true` result sets the event result to `DO_NOT_GROW`. A `false` result leaves its
current result unchanged: the normal `DEFAULT` value continues through vanilla's growth checks,
and a guest never forces growth. If a callback traps or returns a malformed value, only that
module is disabled and the host preserves the current event result.

`BlockGrowAttempt` contains the dimension ID, signed integer block position, and current block
registry ID. These are detached values; no `BlockState` or Java object crosses the ABI. NeoForge's
hook also covers cactus, chorus plants, and other blocks with different growth-state properties,
so the snapshot intentionally omits an age value. Each resource-location string is limited to
256 UTF-8 bytes and the complete schema-v1 payload to 2 KiB.

Schema v1 is little-endian: `u32` version `1`, a stable UTF-8 `dimension_id`, nested schema-v1
`BlockPos` (`i32` `x`, `y`, `z`), then stable UTF-8 `block_id`. Each string is prefixed by its
`i32` byte length. The dimension and block IDs are canonical lowercase resource locations.

The dedicated-server GameTest posts a real `CropGrowEvent.Pre` through NeoForge's event bus with a
known wheat state and position. It verifies the guest receives the detached snapshot and sets
`DO_NOT_GROW`. Posting the pre-event directly keeps the check deterministic instead of waiting for
Minecraft's random-tick scheduling.
