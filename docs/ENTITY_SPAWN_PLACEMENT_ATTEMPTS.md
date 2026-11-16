# Entity spawn placement attempts

Server modules can veto Minecraft's natural spawn placement check before a mob
is created:

```rust
use sjml::entity_spawn_placement_attempt::EntitySpawnPlacementAttempt;
use sjml::{
    event,
    events::{self, EventPriority},
    SjmlResult,
};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("entity.spawn_placement_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.spawn_placement_attempt")]
fn deny_zombie_placements(event: EntitySpawnPlacementAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:zombie")
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's `MobSpawnEvent.SpawnPlacementCheck`, which fires when spawn rules such
as light level, block type, or slime chunks are evaluated. Returning `Ok(true)`
makes the host call `event.setResult(Result.DENY)`, so the placement check is
rejected and the mob never spawns. Returning `Ok(false)` leaves the vanilla
result untouched.

The detached schema-v1 payload contains the canonical entity type ID, the
dimension ID, the block x/y/z of the evaluated position, the lowercase
`MobSpawnType` name (such as `natural` or `spawn_egg`), and the vanilla
`default_result` flag. Strings are bounded to 256 UTF-8 bytes and the payload to
768 bytes. Up to 64 subscribed handlers run per attempt, and all subscriptions
require both `events.subscribe` and `events.cancel`. `#[event_async]` is rejected
because the veto must apply while the placement check is running. Events whose
result is already `SUCCEED`, `FAIL`, or `DENY` are not dispatched to guests.

GameTests post a real `SpawnPlacementCheck` for a zombie and an unmarked cow and
verify that only the zombie placement is denied. Unit tests cover the stable
codec, the lowercase spawn-type vocabulary, schema constants, the NeoForge hook
contract, and detached snapshot values.
