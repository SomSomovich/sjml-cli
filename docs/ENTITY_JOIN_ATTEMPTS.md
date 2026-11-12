# Entity join attempts

`entity.join_attempt` is a synchronous, cancellable event fired before an entity is inserted into
its level. It uses NeoForge 21.1.265's `EntityJoinLevelEvent`, which fires on both logical sides
from the fresh-entity and saved-entity insertion paths. Cancelling the event prevents insertion.

SJML handles only logical-server events on the owning server thread. The bounded schema-v1
snapshot contains the entity UUID, registry type ID, dimension, position, and `loaded_from_disk`
flag. Client-side events and events already cancelled by another listener are ignored. The host
does not query or load chunks from this pre-add hook; callbacks receive detached data and no live
Minecraft references. When a guest vetoes a join, SJML also skips the corresponding
`entity.spawned` notification.

Request both event capabilities in the module and have the server administrator grant them:

```toml
[[modules]]
id = "entity_rules"
path = "modules/entity_rules.wasm"
side = "server"
permissions = ["events.subscribe", "events.cancel"]
```

Subscribe during module initialization and return `Ok(true)` to reject a join:

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.join_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.join_attempt")]
fn reject_area_effect_clouds(event: EntityJoinAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:area_effect_cloud")
}
```

Each string is limited to 256 UTF-8 bytes and the full payload is at most 2 KiB. Position values
must be finite. The callback runs synchronously while the level is processing the pre-add event,
so keep it bounded and avoid work that could load chunks. If a callback fails or returns an invalid
boolean frame, SJML disables that module and preserves the current event result.
