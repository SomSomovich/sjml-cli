# Entity projectile-ammo attempts

Server modules can synchronously veto the ammo stack a living entity selects
for a projectile weapon through `entity.projectile_ammo_attempt`. NeoForge fires
the hook from `LivingEntity#getProjectile(ItemStack)` for the logical server, so
the callback applies before a bow consumes an arrow or a crossbow loads a
projectile. The host dispatches the callback on the logical server's owning
thread; non-server contexts, entities outside a server level, and dedicated or
integrated server mismatches are not dispatched.

The callback requires `events.subscribe` and `events.cancel`. Returning
`Ok(true)` clears the selected ammo, which the host applies by setting the
projectile stack to `ItemStack.EMPTY`; returning `Ok(false)` preserves the
vanilla selection. `#[event_async]` is rejected because the veto must apply
before NeoForge reads the returned projectile.

The detached schema-v1 payload contains the entity UUID, its registry type, and
the dimension ID, followed by two item snapshots in order: `weapon` (the
projectile weapon stack) and `ammo` (the selected projectile stack). Each
snapshot carries a canonical registry item ID and a count. An empty stack is
`minecraft:air` with count 0; `minecraft:air` appears exactly when the count is
0. Counts are bounded to `0..=1,000,000`, IDs must be canonical lowercase
resource locations, strings are bounded to 256 UTF-8 bytes, and the payload to
2 KiB. Up to 64 subscribed handlers run per selection.

```rust
use sjml::{
    entity_projectile_ammo_attempt::EntityProjectileAmmoAttempt,
    event,
    events::{self, EventPriority},
    SjmlResult,
};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("entity.projectile_ammo_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.projectile_ammo_attempt")]
fn veto_bow_ammo(event: EntityProjectileAmmoAttempt) -> SjmlResult<bool> {
    Ok(event.weapon.item_id == "minecraft:bow")
}
```

Only bounded identifiers, item IDs, and counts cross the ABI; the event's
mutable weapon and projectile stacks are left untouched, so a callback can
neither create nor duplicate items. Guest errors disable the failing module and
leave the vanilla ammo selection unchanged.

GameTests spawn a zombie, post a real `LivingGetProjectileEvent` with a bow and
an arrow, and verify that the selected projectile is cleared. A crossbow control
keeps the arrow. Unit tests cover the stable codec, bounds, the non-cancellable
NeoForge contract, and detached snapshot values.
