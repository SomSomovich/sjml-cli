# Cancellable entity target attempts

`entity.target_attempt` is delivered synchronously when a living mob is about to change its
attack target. Return `Ok(true)` to keep the mob's current target, or `Ok(false)` to let the
target change continue:

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.target_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.target_attempt")]
fn protect_villagers(event: EntityTargetAttempt) -> SjmlResult<bool> {
    Ok(event.target_change_type.ends_with("#MOB_TARGET")
        && event.requested_target_id.is_some())
}
```

The module must request and receive both `events.subscribe` and `events.cancel`. The callback uses
synchronous `#[event]`; `#[event_async]` is rejected because the return value controls the live
NeoForge event. Any authorized handler returning `true` vetoes the target change. A callback trap
or malformed result disables only that module and does not veto the change. Events already
canceled by another listener are ignored.

For NeoForge 21.1.265, the host listens to `LivingChangeTargetEvent`. It implements
`ICancellableEvent`, fires before `LivingSetAttackTargetEvent`, and cancellation leaves the
entity's existing target unchanged while preventing `LivingSetAttackTargetEvent` from being
posted. NeoForge calls the hook from both `Mob#setTarget` (`MOB_TARGET`) and the
`StartAttacking` behavior (`BEHAVIOR_TARGET`). A requested `None` represents an attempt to clear
the target.

`EntityTargetAttempt` contains only the mob UUID, optional requested target UUID, and a detached
target-change type string. Built-in enum types are encoded as their fully-qualified Java enum
class plus `#` and the enum name; other target types use their fully-qualified implementation
class name. The host validates the string encoding and limits each string to 65,536 UTF-8 bytes
and the full payload to 1 MiB. Dispatch is restricted to the owning logical-server thread in
dedicated and integrated servers; no Minecraft object crosses into guest code.

Schema v1 is little-endian: `u32` version, length-prefixed UTF-8 `entity_id`, one-byte option tag
and (when present) length-prefixed UTF-8 `requested_target_id`, then length-prefixed UTF-8
`target_change_type`. Each string length uses an `i32`; an option tag of `0` means no requested
target, and `1` is followed by its UUID string.
