# Animal taming attempts

Server modules can veto a vanilla animal taming attempt before the animal is
marked as tamed:

```rust
use sjml::{event, events::{self, EntityTameAttempt, EventPriority}, SjmlResult};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("entity.tame_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.tame_attempt")]
fn block_wolf_taming(event: EntityTameAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:wolf")
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's cancellable `AnimalTameEvent`. Returning `Ok(true)` prevents the
taming from happening: the animal keeps its previous tame and owner state.
Returning `Ok(false)` leaves the vanilla result unchanged. NeoForge fires the
hook from vanilla wolves, cats, ocelots, parrots, and horses; other mods may
fire it for their own animals.

The detached schema-v1 payload contains the animal UUID, its canonical entity
type ID, the dimension ID, the taming player's UUID, and the player's
game-profile name. Each string is bounded to 256 UTF-8 bytes (names to 64), IDs
must be canonical lowercase resource locations, and the payload is at most
2 KiB. No Minecraft objects, inventories, or NBT are exposed. Up to 64
subscribed handlers run per attempt, and all subscriptions require both
`events.subscribe` and `events.cancel`. `#[event_async]` is rejected because the
veto decision must be applied before the taming hook returns.

GameTests post a real `AnimalTameEvent` for a spawned wolf and cow and verify
that the guest veto blocks only the wolf while the cow attempt keeps its vanilla
result. Unit tests cover the stable codec, bounds, the cancellable NeoForge
contract, and detached snapshot values.
