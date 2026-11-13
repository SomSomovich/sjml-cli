# Entity knockback attempts

A module can synchronously observe and veto a living entity's impending
knockback with `entity.knockback_attempt`. The host listens to NeoForge
21.1.265 `LivingKnockBackEvent` on the logical server. The callback runs on the
owning server thread before Minecraft applies knockback velocity.

Subscribe during module initialization and use a synchronous event callback:

```rust
use sjml::{event, events::{self, EventPriority}, mod_init, EntityKnockbackAttempt, SjmlResult};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("entity.knockback_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.knockback_attempt")]
fn guard_knockback(event: EntityKnockbackAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:pig")
}
```

Returning `Ok(true)` cancels knockback; `Ok(false)` leaves it unchanged. The
schema-v1 payload contains the target entity UUID, type ID, dimension ID,
position, original/current strength, and original/current X/Z ratios. Values
are detached and read-only; the guest cannot rewrite strength or direction.
Only finite values within the ABI bounds are delivered. The callback requires
both `events.subscribe` and `events.cancel`. It does not cause chunk loads.

NeoForge fires this hook from damage, shield blocking, Mob attack, and player
attack paths. It represents a knockback attempt; a separate damage callback
may also be delivered for the same gameplay action.
