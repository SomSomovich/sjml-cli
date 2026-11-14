# World save observations

`world.save_observed` reports a logical-server level save using a detached
schema-v1 `WorldSaveObserved` value. It contains only the canonical dimension
resource location and the level's game time. The callback receives no `Level`,
chunk, entity, or NBT object.

NeoForge 21.1.265 fires `LevelEvent.Save` after `ServerLevel#save` writes level
data and asks the chunk and entity storage managers to save. On a flush save,
the event is posted before NeoForge waits for the IO worker. It therefore means
“the save was observed after serialization was requested,” not “all data is
durable.” It can occur more than once for a dimension, including autosaves,
explicit save commands, and shutdown saves.

SJML copies the dimension ID and game time during the event, then queues the
detached values for a later logical-server tick. Guest code never runs from the
save hook and cannot change, cancel, or delay the save that triggered the
notification. The queue holds at most 128 pending observations and drains at
most 32 per server tick; excess observations are dropped. A stopped server
discards its pending observations.

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("world.save_observed", EventPriority::Normal)
}

#[event_async(name = "world.save_observed")]
fn on_world_save(event: WorldSaveObserved) -> SjmlResult<()> {
    host::log(
        LogLevel::Debug,
        &format!("save observed for {} at tick {}", event.dimension_id, event.game_time),
    )
}
```

The event is non-cancellable and requires `events.subscribe`. Game time is
bounded to `0..=1_000_000_000_000` ticks, and the dimension ID is a canonical
resource location of at most 256 UTF-8 bytes. Use the existing world APIs for
any additional reads; this event does not expose or load a world.
