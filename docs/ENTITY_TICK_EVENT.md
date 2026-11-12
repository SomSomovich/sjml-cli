# Entity tick event

Server modules can receive an observation after a loaded non-player entity
completes its logical-server tick. The host sends a detached `EntityTick`
snapshot; it does not expose a Minecraft entity reference.

```rust
use sjml::{event, events::{self, EntityTick, EventPriority}, mod_init, SjmlResult};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe_with_interval("entity.tick", EventPriority::Normal, 20)?;
    Ok(())
}

#[event(name = "entity.tick")]
fn on_entity_tick(event: EntityTick) -> SjmlResult<()> {
    // Read the entity ID, type, dimension, position and entity-local tick count.
    let _ = (event.entity_id, event.entity_type_id, event.dimension_id,
        event.x, event.y, event.z, event.tick_count);
    Ok(())
}
```

The callback requires `events.subscribe` and runs on the owning logical-server
thread. It excludes player entities, runs after `EntityTickEvent.Post`, cannot
cancel or modify the tick, and never loads chunks. Its schema-v1 fields are the
canonical entity UUID, registry type ID, dimension ID, finite X/Y/Z coordinates,
and a non-negative entity-local tick count.

The plain subscription cadence is every 20 entity ticks. Use
`subscribe_with_interval` to choose 1–1,200 entity ticks. The host caps total
`entity.tick` guest callback invocations at 4,096 per logical-server tick;
excess callbacks are skipped until the next tick. The regular per-module
subscription limit also applies.
