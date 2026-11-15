# `player.flyable_fall`

`player.flyable_fall` reports that a flying `ServerPlayer` fell on the logical
server. SJML listens to NeoForge 21.1.265's `PlayerFlyableFallEvent`, which
Minecraft posts when a player is able to fly, for example while gliding, and
still accumulates fall distance. The event is notification-only: it is not an
`ICancellableEvent`, and although the NeoForge hook exposes value setters, SJML
never calls them, so a callback cannot change or cancel the fall.

The notification contains the player's UUID, the player's dimension, the
current fall distance, and the current damage multiplier. The guest receives no
`Player`, level, or other Minecraft object.

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
delivered on the owning server thread; a client-only module does not receive
it. Because the handler cannot veto, a `#[event_async]` handler is also
allowed and receives the same immutable snapshot.

Each module may hold up to 256 event subscriptions. For one fall, the host
invokes at most 64 subscribed callbacks. The complete payload is limited to
768 bytes, and each string is limited to 256 UTF-8 bytes.

## Rust handler

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("player.flyable_fall", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.flyable_fall")]
fn flyable_fall(event: PlayerFlyableFall) -> SjmlResult<()> {
    host::log(
        LogLevel::Info,
        &format!(
            "{} is falling in {} (distance={}, multiplier={})",
            event.player_id, event.dimension_id, event.distance, event.multiplier,
        ),
    )?;
    Ok(())
}
```

The compilable SDK example is
[`player_flyable_fall.rs`](../sjml-cli/sjml-sdk/examples/gametest/player_flyable_fall.rs).

## Schema v1

The callback export is `sjml_v1_event_player.flyable_fall`. Its payload fields
use the stable little-endian encoding:

1. `u32(1)` schema version.
2. Length-prefixed UTF-8 canonical player UUID.
3. Length-prefixed UTF-8 canonical dimension resource location.
4. `f32` `distance`: the current fall distance, finite and within `0..=10,000`.
5. `f32` `multiplier`: the current fall damage multiplier, finite and within
   `0..=10,000`.

All strings use a little-endian `u32` byte length followed by UTF-8 bytes.
