# Potential spawns attempts

Server modules can veto the natural spawner's candidate list at a position
before any mob is chosen:

```rust
use sjml::world_potential_spawns_attempt::WorldPotentialSpawnsAttempt;
use sjml::{event, events::{self, EventPriority}, SjmlResult};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("world.potential_spawns_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "world.potential_spawns_attempt")]
fn block_monster_candidates(event: WorldPotentialSpawnsAttempt) -> SjmlResult<bool> {
    Ok(event.category == "monster")
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's cancellable `LevelEvent.PotentialSpawns`, which Minecraft posts when
the natural spawner builds its candidate list for a position. Returning
`Ok(true)` calls `event.setCanceled(true)`, so the candidate list becomes empty
and no natural entity spawns at that position. Returning `Ok(false)` keeps
vanilla candidates and behavior. The host only dispatches for integrated or
dedicated server contexts, ignores events already canceled by another handler,
and rejects an off-thread capture.

The detached schema-v1 payload contains, in order: the dimension ID, the
lowercase `MobCategory` name (such as `monster` or `creature`), the block
position `x`, `y`, `z` as `i32`, and the candidate count. The candidate count
must be within `0..=4096`. The dimension must be a canonical lowercase resource
location; strings are bounded to 256 UTF-8 bytes, the category name to 64 bytes,
and the payload to 768 bytes. Up to 64 subscribed handlers run per attempt, and
all subscriptions require both `events.subscribe` and `events.cancel`.
`#[event_async]` is rejected because the veto must apply before the spawner
chooses a mob.

GameTests post a real `LevelEvent.PotentialSpawns` for a `monster` category and
a `creature` control, then verify that the guest cancels only the marked
category. Unit tests cover the stable codec, the category vocabulary, candidate
bounds, and the cancellable NeoForge contract.
