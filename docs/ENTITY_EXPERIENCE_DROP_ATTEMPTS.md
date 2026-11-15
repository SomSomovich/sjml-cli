# Experience-drop attempts

Server modules can veto the experience orbs produced by a living entity's
death:

```rust
use sjml::{event, events::{self, EntityExperienceDropAttempt, EventPriority}, SjmlResult};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("entity.experience_drop_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.experience_drop_attempt")]
fn no_experience_for_pigs(event: EntityExperienceDropAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:pig")
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's cancellable `LivingExperienceDropEvent`. Returning `Ok(true)` makes
Minecraft award zero experience; the entity still dies and item drops remain
controlled by the separate `entity.drops_attempt` event. Returning `Ok(false)`
leaves the vanilla result unchanged. Minecraft fires the hook only when a
death would otherwise drop experience, so it does not fire for passive deaths
that already produce no orbs.

The detached schema-v1 payload contains the entity UUID, canonical entity type
ID, dimension ID, an optional UUID for the last attacking player, and the
original and currently requested experience amounts. Each experience value is
bounded to `0..=1,000,000`; strings are bounded to 256 UTF-8 bytes and IDs must
be canonical lowercase resource locations. No Minecraft objects, damage
sources, or item stacks are exposed. Up to 64 subscribed handlers run per
attempt, and all subscriptions require both `events.subscribe` and
`events.cancel`. `#[event_async]` is rejected because the veto decision must be
applied before the death hook returns.

GameTests post a real `LivingExperienceDropEvent` for a spawned pig and verify
that the guest veto cancels only the marked amount while an unmarked amount
keeps its vanilla result. Unit tests cover the stable codec, bounds, the
cancellable NeoForge contract, and detached snapshot values.
