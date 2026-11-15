# `entity.breathe`

`entity.breathe` reports how a living entity resolves its air supply during one
server tick. SJML listens to NeoForge 21.1.265's `LivingBreatheEvent`, which
Minecraft posts once per living-entity tick on the logical server. The event is
notification-only: it is not an `ICancellableEvent`, and the host captures the
values without writing them back, so a callback cannot change or cancel air
consumption or refill even though NeoForge exposes setters.

The notification contains the entity's UUID, the entity type registry ID, the
entity's dimension, and the three values `can_breathe`, `consume_air`, and
`refill_air`. The guest receives no `LivingEntity`, level, or other Minecraft
object.

## Permissions and execution

Declare the subscription permission in the module manifest:

```toml
permissions = ["events.subscribe"]
```

The server operator must grant it to the module as well:

```toml
[[grant]]
module = "example_mod/main"
capabilities = ["events.subscribe"]
```

The event only runs in an integrated or dedicated server context. It is
dispatched on the owning logical-server thread; a client-only module does not
receive it. Because the handler cannot veto or mutate, an `#[event_async]`
handler is also allowed and receives the same immutable snapshot.

Each module may hold up to 256 event subscriptions. For one breath tick, the
host invokes at most 64 subscribed callbacks. The complete payload is limited to
768 bytes, and each string is limited to 256 UTF-8 bytes. `consume_air` and
`refill_air` are bounded to 0 through 1,000,000.

## Rust handler

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.breathe", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.breathe")]
fn breathe(event: EntityBreathe) -> SjmlResult<()> {
    host::log(
        LogLevel::Info,
        &format!(
            "{} ({}) in {} can_breathe={} consume={} refill={}",
            event.entity_id,
            event.entity_type_id,
            event.dimension_id,
            event.can_breathe,
            event.consume_air,
            event.refill_air,
        ),
    )?;
    Ok(())
}
```

The compilable SDK example is
[`entity_breathe.rs`](../sjml-cli/sjml-sdk/examples/gametest/entity_breathe.rs).

## Schema v1

The callback export is `sjml_v1_event_entity.breathe`. Its payload fields use
the stable little-endian encoding:

1. `i32(1)` schema version.
2. Length-prefixed UTF-8 canonical entity UUID.
3. Length-prefixed UTF-8 canonical entity type resource location.
4. Length-prefixed UTF-8 canonical dimension resource location.
5. `u8` `can_breathe`: `1` when the entity can breathe this tick.
6. `i32` `consume_air`: air units removed when the entity cannot breathe.
7. `i32` `refill_air`: air units restored when the entity can breathe.

All strings use a little-endian `u32` byte length followed by UTF-8 bytes.
