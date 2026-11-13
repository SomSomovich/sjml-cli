# World game-event dispatch

Server modules can dispatch a registered vanilla or modded Minecraft game event at a position in an already-loaded world:

```rust
use sjml::prelude::*;

fn notify_sculk(world: WorldHandle, position: BlockPos) -> SjmlResult<()> {
    world::emit_game_event(world, position, "minecraft:step")
}
```

The module must request and receive both `world.read` and `world.write`. `event_id` must be a canonical lowercase `namespace:path` key in the selected level's current `Registries.GAME_EVENT` registry. The host dispatches it at the center of the requested block with an empty source context: guests cannot claim that a player, entity, or block produced the event.

Calls run synchronously on the owning logical-server thread. The position must fit the dimension's build height and world border, and the chunk must already be loaded. SJML never requests a chunk load. Each module can emit at most 32 events per logical-server tick. Static-registration and client-side calls are denied.

The event uses Minecraft's regular listener pipeline, so sculk sensors and other vanilla or modded listeners react according to the selected event type. The guest does not receive a guarantee that a particular listener handled it. The dedicated-server GameTest dispatches `minecraft:step` from a guest and checks that a nearby sculk sensor activates while an unloaded remote target stays unloaded.

The synchronous `sjml_world_v1.emit_game_event(query_ptr, query_len)` import accepts schema-v1 data bounded to 304 bytes. Its exact field layout is in [ABI.md](ABI.md#logical-server-worlds).
