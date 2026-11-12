# Projectile impact attempt

Use `entity.projectile_impact_attempt` to inspect or veto a server-side projectile impact without
receiving Minecraft Java objects:

```rust
use sjml::prelude::*;

#[event(name = "entity.projectile_impact_attempt")]
fn cancel_stone_impact(event: ProjectileImpactAttempt) -> SjmlResult<bool> {
    Ok(event.hit_kind == "block"
        && event.hit_block_id.as_deref() == Some("minecraft:stone"))
}

fn register_events() -> SjmlResult<()> {
    events::subscribe("entity.projectile_impact_attempt", EventPriority::NORMAL)?;
    Ok(())
}
```

Grant both `events.subscribe` and `events.cancel` to the module. The callback runs synchronously on
the owning server thread. Return `Ok(true)` to cancel the hit; NeoForge leaves the projectile flying
when its impact event is cancelled. `hit_kind` is `entity` or `block`. Entity hits set
`hit_entity_id`/`hit_entity_type_id`; block hits set `hit_block_id`, `block_position`, and
`block_face`. All other optional target fields are `None`. `x`, `y`, and `z` are the ray's impact
coordinates.

The host ignores client prediction, already-cancelled events, unsupported hit kinds, and unknown
registry IDs. It checks that a block-hit chunk is already loaded before reading the block ID, so this
event never loads chunks. The schema is bounded to 4 KiB with at most 256 UTF-8 bytes per string.
The pinned NeoForge 21.1.265 `ProjectileImpactEvent` implements `ICancellableEvent`; the vanilla
hook posts it for arrows, fireballs, fireworks, fishing hooks, llama spit, shulker bullets, and
throwable projectiles. Custom projectiles need to post the same NeoForge event to be covered.
