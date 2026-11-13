# Living-entity drop attempts

The synchronous logical-server `entity.drops_attempt` event is backed by
NeoForge 21.1.265's cancellable `LivingDropsEvent`. It runs after the entity's
death loot is computed and before the listed item entities are spawned. A
handler returning `Ok(true)` cancels that event's item drops; the entity still
dies.

```rust
use sjml::{event, events::{self, EntityDropsAttempt, EventPriority}, SjmlResult};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("entity.drops_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.drops_attempt")]
fn keep_pig_drops(event: EntityDropsAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:pig"
        && event.drops.iter().any(|drop| drop.item_id == "minecraft:porkchop"))
}
```

The schema-v1 snapshot carries the entity UUID/type ID, dimension ID, a bounded
list of item IDs/counts/optional durability, `recently_hit`, and
`drops_truncated`. It omits the damage source, item components, NBT, and live
Minecraft objects. The item list contains at most 32 entries, the complete
payload is capped at 16 KiB, and each item count/durability is bounded to
1,000,000. A truncated snapshot still represents the full `LivingDropsEvent`
for cancellation: returning true cancels the event's entire item-drop list.
The callback does not control entity death or expose experience as an action.

At most 64 subscribed callbacks run synchronously on the owning logical-server
thread. Both `events.subscribe` and `events.cancel` are required; client modules
cannot subscribe and `#[event_async]` is rejected. Already-canceled NeoForge
events are skipped. If detached data cannot be encoded, the host lets the
vanilla drop flow continue.

[`EntityDropsAttemptEventAdapterTest`](../src/test/java/dev/somuch/sjml/neoforge/event/EntityDropsAttemptEventAdapterTest.java)
checks the pinned NeoForge event contract and that guest snapshots contain no
Minecraft object references. [`EntityDropsAttemptGameTests`](../src/main/java/dev/somuch/sjml/neoforge/gametest/EntityDropsAttemptGameTests.java)
vetoes the guest-selected pig drop and checks that a second pig retains its
vanilla porkchop drops.
