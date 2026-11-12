# Entity heal attempts

`entity.heal_attempt` is a synchronous, cancellable event for a living entity
that is about to be healed. It uses NeoForge 21.1.265's `LivingHealEvent`,
which NeoForge fires from `LivingEntity.heal(float)` before applying the heal.
The event is cancellable, and cancelling it prevents the health increase.

The host only dispatches the callback on the logical server's owning thread.
Client-side prediction, already-cancelled events, and calls without an active
server runtime are ignored. The callback receives a detached snapshot with the
entity UUID and registry type ID, dimension ID, position, and current heal
amount. It receives no Minecraft object references and cannot change the heal
amount.

Request both event capabilities in the module and have the server administrator
grant them:

```toml
[[modules]]
id = "healing_rules"
path = "modules/healing_rules.wasm"
side = "server"
permissions = ["events.subscribe", "events.cancel"]
```

Subscribe during module initialization and return `Ok(true)` to veto a heal:

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.heal_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.heal_attempt")]
fn prevent_large_pig_heals(event: EntityHealAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:pig" && event.amount >= 4.0)
}
```

`EntityHealAttempt.amount` is finite and bounded to `0..=1_000_000`; event
identifiers are UTF-8 strings of at most 256 bytes, and the whole schema-v1
payload is at most 2 KiB. The guest callback runs synchronously before
`LivingHealEvent` returns, so it should do only bounded work. If a callback
fails or returns an invalid boolean frame, the host disables that module and
allows vanilla healing to continue.
