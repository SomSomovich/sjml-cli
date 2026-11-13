# Farmland trampling attempts

Logical-server modules can synchronously veto NeoForge's cancellable
`BlockEvent.FarmlandTrampleEvent`. A `true` result prevents the farmland from
degrading for that attempt; `false` preserves the event's existing result.
The host skips an event another listener has already canceled. Subscribe from a
retained server module and request both `events.subscribe` and `events.cancel`.
The handler must use `#[event]`; asynchronous callbacks cannot return a veto in
time.

The `BlockFarmlandTrampleAttempt` snapshot contains the dimension ID, position,
block ID, entity UUID, entity type ID, and fall distance. It holds no
Minecraft object references and omits block-state properties. The event is
captured on the owning logical-server thread. Dimension, block, and entity
type IDs must be canonical lowercase resource locations. Strings are limited
to 256 UTF-8 bytes, the UUID must be canonical, fall distance must be
finite and within `0..=10_000`, and the complete schema-v1 payload is capped at
2 KiB. Guest callbacks are capped at 64 for one event.

```rust
use sjml::{event, events::BlockFarmlandTrampleAttempt, SjmlResult};

#[event(name = "block.farmland_trample_attempt")]
fn preserve_farmland(event: BlockFarmlandTrampleAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:pig")
}
```

Subscribe with `events::subscribe("block.farmland_trample_attempt", priority)`
from `mod_init`. This hook only decides whether the current eligible landing
tramples farmland; it does not prevent the entity from landing or change other
block behavior.
