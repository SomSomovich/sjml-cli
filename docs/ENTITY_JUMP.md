# `entity.jump`

`entity.jump` reports that a living entity jumped on the logical server. SJML
listens to NeoForge 21.1.265's `LivingEvent.LivingJumpEvent`, which Minecraft
posts from `LivingEntity#jumpFromGround` and the corresponding slime, magma cube,
camel, and horse jump paths. The event is notification-only: it is not an
`ICancellableEvent` and exposes no result or setter, so a callback cannot change
or cancel the jump.

The notification contains the entity's UUID, the entity type registry ID, and
the entity's dimension. The guest receives no `LivingEntity`, level, or other
Minecraft object.

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
receive it. Because the handler cannot veto, a `#[event_async]` handler is also
allowed and receives the same immutable snapshot.

Each module may hold up to 256 event subscriptions. For one jump, the host
invokes at most 64 subscribed callbacks. The complete payload is limited to
512 bytes, and each string is limited to 256 UTF-8 bytes.

## Rust handler

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.jump", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.jump")]
fn jump(event: EntityJump) -> SjmlResult<()> {
    host::log(
        LogLevel::Info,
        &format!(
            "{} ({}) jumped in {}",
            event.entity_id, event.entity_type_id, event.dimension_id,
        ),
    )?;
    Ok(())
}
```

The compilable SDK example is
[`entity_jump.rs`](../sjml-cli/sjml-sdk/examples/gametest/entity_jump.rs).

## Schema v1

The callback export is `sjml_v1_event_entity.jump`. Its payload fields use the
stable little-endian encoding:

1. `i32(1)` schema version.
2. Length-prefixed UTF-8 canonical entity UUID.
3. Length-prefixed UTF-8 canonical entity type resource location.
4. Length-prefixed UTF-8 canonical dimension resource location.

All strings use a little-endian `u32` byte length followed by UTF-8 bytes.
