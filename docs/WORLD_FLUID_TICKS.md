# Scheduled fluid ticks

Server-side Rust modules can schedule a normal Minecraft fluid tick at a position that already contains the expected registered fluid:

```rust
let overworld = world::get("minecraft:overworld")?.ok_or_else(|| {
    SjmlError::new(ErrorCode::InvalidArgument, "the Overworld is not loaded")
})?;
world::schedule_fluid_tick(
    overworld,
    BlockPos { x: 4, y: 64, z: -3 },
    "minecraft:water",
    20,
)?;
```

The module must request and receive `world.read`, `world.write`, and `world.fluid_tick.schedule`. The scheduler validates a canonical fluid registry ID, a delay from 1 through 72,000 ticks, the current fluid at the position, build height, and the world border. The target chunk must already be loaded; this API never loads it. A module can request at most 16 fluid ticks during one logical-server tick. Minecraft applies its normal fluid tick de-duplication and execution behavior.

This operation schedules Minecraft's fluid update callback. It does not call guest code directly. Block tick callbacks are a separate API documented in [`BLOCK_TICK.md`](BLOCK_TICK.md).
