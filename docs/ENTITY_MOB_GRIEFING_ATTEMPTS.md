# Mob griefing attempts

Server modules can deny a mob's griefing action before vanilla applies it:

```rust
use sjml::{
    entity_mob_griefing_attempt::EntityMobGriefingAttempt, event,
    events::{self, EventPriority}, SjmlResult,
};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("entity.mob_griefing_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.mob_griefing_attempt")]
fn stop_zombie_griefing(event: EntityMobGriefingAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:zombie")
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's `EntityMobGriefingEvent`, which vanilla consults whenever it checks
the `mobGriefing` game rule. Returning `Ok(true)` means the entity may not
perform the griefing action: the host calls `setCanGrief(false)`, so the
attempting behavior, such as an enderman picking up a block or a zombie breaking
a door, is skipped. Returning `Ok(false)` keeps vanilla behavior. The event is
not cancellable and has no cancel state; the veto is the `canGrief` flag alone.
A guest can only deny the action. When the game rule is disabled, vanilla
already reports `canGrief == false`, and `Ok(false)` cannot re-enable griefing.

The detached schema-v1 payload contains the entity's canonical UUID, its
registry type ID, the dimension ID, and a `griefing_enabled` flag carrying
`isMobGriefingEnabled()`, which is the game rule value at capture time. IDs must
be canonical lowercase resource locations; strings are bounded to 256 UTF-8
bytes and the payload to 1 KiB. Up to 64 subscribed handlers run per attempt,
and all subscriptions require both `events.subscribe` and `events.cancel`.
`#[event_async]` is rejected because the veto must apply before the griefing
action proceeds. Attempts observed outside a logical server, such as client
levels, and dedicated/integrated context mismatches are ignored without
invoking guests.

GameTests post a real `EntityMobGriefingEvent` for a synthesized zombie and a
cow control and verify that only the zombie loses its griefing permission. Unit
tests cover the stable codec, bounds, the non-cancellable NeoForge contract, and
detached snapshot values.
