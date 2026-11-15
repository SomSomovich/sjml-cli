# Mob split attempts

Server modules can veto a mob split before any children spawn:

```rust
use sjml::{event, events::{self, EntityMobSplitAttempt, EventPriority}, SjmlResult};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("entity.mob_split_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.mob_split_attempt")]
fn stop_slime_split(event: EntityMobSplitAttempt) -> SjmlResult<bool> {
    Ok(event.parent_type_id == "minecraft:slime" && event.child_count > 3)
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's cancellable `MobSplitEvent`. In vanilla it fires for slimes and magma
cubes when they are removed; mods may fire it for their own mobs. Returning
`Ok(true)` prevents every child from spawning; returning `Ok(false)` leaves the
vanilla child list unchanged.

The detached schema-v1 payload contains the parent mob UUID and registry
type, the dimension ID, and the number of children that would spawn
(`0..=1,000,000`). IDs must be canonical lowercase resource locations; strings
are bounded to 256 UTF-8 bytes and the payload to 2 KiB. Up to 64 subscribed
handlers run per attempt, and all subscriptions require both
`events.subscribe` and `events.cancel`. `#[event_async]` is rejected because the
veto must apply before children are added.

GameTests post a real `MobSplitEvent` for a synthesized slime and a cow control
and verify that only the slime split is canceled. Unit tests cover the stable
codec, bounds, the cancellable NeoForge contract, and detached snapshot values.
