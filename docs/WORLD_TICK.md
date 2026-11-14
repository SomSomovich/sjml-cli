# World tick notifications

`world.tick` reports each loaded logical-server level after that level finishes
its tick. The detached schema-v1 `WorldTick` contains the canonical dimension
ID, the level's game time, and its day time. It exposes no Minecraft objects
and does not load chunks.

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe_with_interval("world.tick", EventPriority::Normal, 20)?;
    Ok(())
}

#[event(name = "world.tick")]
fn on_world_tick(event: WorldTick) -> SjmlResult<()> {
    if event.dimension_id == "minecraft:overworld" {
        host::log(LogLevel::Debug, &format!("world time: {}", event.day_time))?;
    }
    Ok(())
}
```

The default cadence is 20 ticks. `subscribe_with_interval` accepts 1–1,200
ticks; the host tests the world's game time against that interval. The event
requires `events.subscribe`, runs only in a logical-server context, and is
non-cancellable. Each callback is routed to its owning module. The event does
not provide a world handle, so use the existing loaded-world APIs for any
additional reads or writes.
