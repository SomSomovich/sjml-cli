# Entity effect application attempts

`entity.effect_application` is a synchronous logical-server event for a status effect that is being
checked for application to a living entity. It uses NeoForge 21.1.265's
`MobEffectEvent.Applicable`, fired whenever `LivingEntity.canBeAffected(MobEffectInstance)` is
checked. Returning `Ok(true)` sets the event result to `DO_NOT_APPLY`; an earlier
`DO_NOT_APPLY` result is preserved and is not sent to SJML handlers.

The host dispatches on the owning server thread at the lowest NeoForge listener priority. Client
checks and events without an active server runtime are ignored. The bounded schema-v1 snapshot
contains the affected entity UUID, type, dimension and position, effect registry ID, duration and
amplifier, and optional source entity UUID and type. No live Minecraft objects cross into guest
code. The callback cannot modify the effect values.

Request both event capabilities in the module and have the server administrator grant them:

```toml
[[modules]]
id = "effect_rules"
path = "modules/effect_rules.wasm"
side = "server"
permissions = ["events.subscribe", "events.cancel"]
```

Subscribe during module initialization and return `Ok(true)` to reject an application:

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.effect_application", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.effect_application")]
fn reject_poison_on_cows(event: EntityEffectApplicationAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:cow" && event.effect_id == "minecraft:poison")
}
```

Every string is limited to 256 UTF-8 bytes, and the complete payload is at most 2 KiB. Effect
duration is bounded by its signed 32-bit representation (`-1` means infinite); amplifier is a
non-negative signed 32-bit value. The event callback runs synchronously inside NeoForge's
applicability check, so it should do only bounded work. Callback failures and invalid boolean
frames disable the faulty module and leave the current event result in place.
