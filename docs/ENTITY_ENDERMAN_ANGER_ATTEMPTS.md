# Enderman anger attempts

Server modules can stop an enderman from targeting a player:

```rust
use sjml::{
    event,
    events::{self, EventPriority},
    mod_init, EntityEndermanAngerAttempt, SjmlResult,
};

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.enderman_anger_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.enderman_anger_attempt")]
fn calm_endermen(event: EntityEndermanAngerAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:enderman")
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's cancellable `EnderManAngerEvent`, fired before an enderman detects
that a player is looking at it. Returning `Ok(true)` cancels the event, so the
enderman does not target the player; returning `Ok(false)` keeps vanilla
targeting. NeoForge does not fire the event when detection is already prevented,
for example by a carved pumpkin, so a veto only applies to attempts vanilla
would otherwise allow.

The detached schema-v1 payload contains the canonical enderman UUID, its
registry type ID, the canonical target player UUID, and the dimension ID. UUIDs
must be canonical and IDs must be lowercase resource locations; strings are
bounded to 256 UTF-8 bytes and the payload to 1 KiB. Up to 64 subscribed
handlers run per attempt, and all subscriptions require both `events.subscribe`
and `events.cancel`. `#[event_async]` is rejected because the veto must apply
before the enderman reads the result.

The host ignores events that are already canceled, events outside a logical
server context, endermen that are not in a `ServerLevel`, and dedicated or
integrated context mismatches. Capture must happen on the owning server thread;
an attempt observed elsewhere is not dispatched to guests.

Unit tests cover the stable codec, bounds, the cancellable NeoForge contract,
and detached snapshot values. GameTests post a real `EnderManAngerEvent` for a
synthesized enderman and a mock server player and verify that the guest veto
cancels it.
