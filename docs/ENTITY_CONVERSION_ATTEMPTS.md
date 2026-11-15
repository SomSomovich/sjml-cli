# Entity conversion attempts

Server modules can veto a living entity's conversion into another entity type
before the replacement spawns:

```rust
use sjml::{event, events::{self, EntityConvertAttempt, EventPriority}, SjmlResult};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("entity.convert_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.convert_attempt")]
fn keep_pigs_as_pigs(event: EntityConvertAttempt) -> SjmlResult<bool> {
    Ok(event.entity_type_id == "minecraft:pig"
        && event.outcome_entity_type_id == "minecraft:zombified_piglin")
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's cancellable `LivingConversionEvent.Pre`. Returning `Ok(true)` keeps
the original entity alive; returning `Ok(false)` leaves the vanilla conversion
unchanged. NeoForge fires the hook from lightning-driven pig and villager
conversions, zombie-villager curing, and mod-defined conversions that call
`EventHooks.canLivingConvert`. Some entities retry the conversion every tick,
so a persistent veto also keeps firing until the conversion timer changes.

The detached schema-v1 payload contains the source entity UUID, its canonical
entity type ID, the dimension ID, and the canonical outcome entity type ID.
Every string is bounded to 256 UTF-8 bytes, IDs must be canonical lowercase
resource locations, and the payload is at most 2 KiB. No Minecraft objects,
timers, or inventory data are exposed. Up to 64 subscribed handlers run per
attempt, and all subscriptions require both `events.subscribe` and
`events.cancel`. `#[event_async]` is rejected because the veto decision must be
applied before the conversion hook returns.

GameTests post a real `LivingConversionEvent.Pre` for a spawned pig and sheep
and verify that the guest veto blocks only the pig-to-zombified-piglin
conversion while the unmarked attempt keeps its vanilla result. Unit tests
cover the stable codec, bounds, the cancellable NeoForge contract, and detached
snapshot values.
