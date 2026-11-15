# Entity block destruction attempts

Server modules can veto a living entity's attempt to break a block before
Minecraft destroys it:

```rust
use sjml::{
    event,
    events::{self, EventPriority},
    mod_init, EntityDestroyBlockAttempt, SjmlResult,
};

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.destroy_block_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.destroy_block_attempt")]
fn keep_zombie_doors(event: EntityDestroyBlockAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:zombie")
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's cancellable `LivingDestroyBlockEvent`, fired when an ender dragon or
wither attempts to destroy a block and when a zombie attempts to break a door.
Returning `Ok(true)` cancels the event, so the block is not destroyed and no
vanilla break behavior runs; returning `Ok(false)` keeps the vanilla result.

The detached schema-v1 payload contains the canonical entity UUID, the entity
type ID, the dimension ID, the target block x/y/z as little-endian `i32`
values, the canonical block ID, and the canonical block state text (at most 512
bytes). Coordinates must be within ±30,000,000, other strings are bounded to
256 UTF-8 bytes, and the payload to 2 KiB. Up to 64 subscribed handlers run per
attempt, and all subscriptions require both `events.subscribe` and
`events.cancel`. `#[event_async]` is rejected because the cancellation must
apply before vanilla destroys the block.

The host ignores already-canceled events, events from non-server contexts, and
entities that are not in a `ServerLevel`; dedicated and integrated server
contexts must match. Unit tests cover the stable codec, the state-size and ID
bounds, the cancellable NeoForge contract, and detached snapshot values.
GameTests post a real `LivingDestroyBlockEvent` for a zombie and a control mob
against an oak door and verify that the guest veto cancels only the zombie
attempt.
