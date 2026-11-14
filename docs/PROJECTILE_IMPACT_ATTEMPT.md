# `entity.projectile_impact_attempt`

`entity.projectile_impact_attempt` is a synchronous, cancellable callback for a server-side
projectile hit. Register a normal `#[event]` handler returning `SjmlResult<bool>` and subscribe with
both `events.subscribe` and `events.cancel`:

```rust
use sjml::{event, events::{self, EventPriority, ProjectileImpactAttempt}, SjmlResult};

#[event(name = "entity.projectile_impact_attempt")]
fn stop_arrows_from_hitting_stone(hit: ProjectileImpactAttempt) -> SjmlResult<bool> {
    Ok(hit.projectile_type_id == "minecraft:arrow"
        && hit.hit_kind == "block"
        && hit.hit_block_id.as_deref() == Some("minecraft:stone"))
}

fn register_events() -> SjmlResult<()> {
    events::subscribe("entity.projectile_impact_attempt", EventPriority::NORMAL)?;
    Ok(())
}
```

Return `Ok(true)` to cancel the impact. In the pinned NeoForge 21.1.265 API,
`ProjectileImpactEvent` implements `ICancellableEvent`, and `EventHooks.onProjectileImpact(Projectile,
HitResult)` returns whether it was cancelled. A cancelled hit is not processed and the projectile
continues flying. SJML dispatches only on the owning server thread and ignores client predictions
and impacts that another listener has already cancelled.

The callback record contains `projectile_id`, `projectile_type_id`, `dimension_id`, `hit_kind`
(`entity` or `block`), the optional target fields, and the hit position (`x`, `y`, `z`). Entity hits
include `hit_entity_id` and `hit_entity_type_id`. Block hits include `hit_block_id`, `block_position`,
and `block_face`. Values are detached strings, UUIDs, coordinates, and integers; the guest receives
no Java object. Each UTF-8 string is limited to 256 bytes and the encoded payload is limited to 4 KiB.

For a block hit, SJML checks `ServerLevel.hasChunkAt` before reading its block ID; it does not load
chunks. Unrecognized hit types, missing registry IDs, invalid payload values, or unloaded block-hit
positions are skipped and vanilla continues. The NeoForge hook is used by vanilla projectile classes
including arrows, fireballs, fireworks, fishing hooks, llama spit, shulker bullets, and throwable
projectiles. A custom projectile must post NeoForge's `ProjectileImpactEvent` itself to participate.
