# Entity armor hurt attempts

Server modules can veto durability damage to a living entity's armor:

```rust
use sjml::entity_armor_hurt_attempt::EntityArmorHurtAttempt;
use sjml::{event, events::{self, EventPriority}, SjmlResult};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("entity.armor_hurt_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.armor_hurt_attempt")]
fn protect_zombie_armor(event: EntityArmorHurtAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:zombie"
        && event.damage_source == "death.attack.generic")
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's cancellable `ArmorHurtEvent`, fired from `LivingEntity.doHurtEquipment`
when a hit would damage one or more equipped armor pieces. Returning `Ok(true)`
calls `event.setCanceled(true)`, so none of the entity's armor loses durability
from that hit. Returning `Ok(false)` leaves vanilla durability loss and the
per-slot adjusted damage untouched. Both `events.subscribe` and `events.cancel`
are required, and `#[event_async]` is rejected because vanilla applies the
durability loss immediately after the event returns.

The detached schema-v1 payload contains, in order: the canonical entity UUID,
the canonical entity type ID, the dimension ID, and the damage source message ID
from `DamageSource.getMsgId()`, such as `death.attack.generic`. IDs must be
canonical lowercase resource locations. The damage source accepts 1 to 64 UTF-8
bytes matching `[a-z0-9_.:/ -]`; other strings are bounded to 256 UTF-8 bytes
and the payload to 1 KiB. Up to 64 subscribed handlers run per attempt.

The host ignores events that are already canceled, events captured outside a
`ServerLevel`, events from non-server contexts, and contexts whose dedicated or
integrated mode does not match the owning server. Capturing off the owning
logical-server thread fails with `IllegalStateException`. GameTests post a real
`ArmorHurtEvent` for a zombie and a cow control and verify that the guest vetoes
only the zombie. Unit tests cover the stable codec, damage-source bounds, the
cancellable NeoForge contract, and detached snapshot values.
