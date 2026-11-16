# `player.changed_dimension`

`player.changed_dimension` reports that a `ServerPlayer` finished changing
dimension on the logical server. SJML listens to NeoForge 21.1.265's
`PlayerEvent.PlayerChangedDimensionEvent`, which the server posts after the
player has left one dimension and entered another. The event is
notification-only: it is not an `ICancellableEvent` and exposes no result or
setter, so a callback cannot change or cancel the completed transfer.

The notification contains the player's UUID and the source and destination
dimension resource locations. The guest receives no `Player`, level,
`ResourceKey`, or other Minecraft object.

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

The event only runs in an integrated or dedicated server context and is
dispatched only on the owning logical-server thread. A client-only module does
not receive it. Because the handler cannot veto, a `#[event_async]` handler is
also allowed and receives the same immutable snapshot.

Each module may hold up to 256 event subscriptions. For one dimension change,
the host invokes at most 64 subscribed callbacks. The complete payload is
limited to 768 bytes, and each string is limited to 256 UTF-8 bytes.

## Rust handler

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("player.changed_dimension", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.changed_dimension")]
fn changed_dimension(event: PlayerChangedDimension) -> SjmlResult<()> {
    host::log(
        LogLevel::Info,
        &format!(
            "{} moved from {} to {}",
            event.player_id, event.from_dimension_id, event.to_dimension_id,
        ),
    )?;
    Ok(())
}
```

The compilable SDK example is
[`player_changed_dimension.rs`](../sjml-cli/sjml-sdk/examples/gametest/player_changed_dimension.rs).

## Schema v1

The callback export is `sjml_v1_event_player.changed_dimension`. Its payload
fields use the stable little-endian encoding:

1. `u32(1)` schema version.
2. Length-prefixed UTF-8 canonical player UUID.
3. Length-prefixed UTF-8 canonical source dimension resource location.
4. Length-prefixed UTF-8 canonical destination dimension resource location.

All strings use a little-endian `u32` byte length followed by UTF-8 bytes.
