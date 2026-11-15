# `player.wake_up`

`player.wake_up` reports that a `ServerPlayer` woke up on the logical server.
SJML listens to NeoForge 21.1.265's `PlayerWakeUpEvent`, which Minecraft posts
after the player leaves a bed. The event is notification-only: it is not an
`ICancellableEvent` and exposes no result or setter, so a callback cannot change
or cancel the wake-up.

The notification contains the player's UUID, the player's dimension, and the
two vanilla flags `wake_immediately()` and `updateLevel()`. The guest receives
no `Player`, level, or other Minecraft object.

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

Each module may hold up to 256 event subscriptions. For one wake-up, the host
invokes at most 64 subscribed callbacks. The complete payload is limited to
1 KiB, and each string is limited to 256 UTF-8 bytes.

## Rust handler

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("player.wake_up", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.wake_up")]
fn wake_up(event: PlayerWakeUp) -> SjmlResult<()> {
    host::log(
        LogLevel::Info,
        &format!(
            "{} woke up in {} (immediate={}, update_level={})",
            event.player_id, event.dimension_id, event.wake_immediately, event.update_level,
        ),
    )?;
    Ok(())
}
```

The compilable SDK example is
[`player_wake_up.rs`](../sjml-cli/sjml-sdk/examples/gametest/player_wake_up.rs).

## Schema v1

The callback export is `sjml_v1_event_player.wake_up`. Its payload fields use
the stable little-endian encoding:

1. `u32(1)` schema version.
2. Length-prefixed UTF-8 canonical player UUID.
3. Length-prefixed UTF-8 canonical dimension resource location.
4. `u8` `wake_immediately`: `1` when the wake-up animation is skipped.
5. `u8` `update_level`: `1` when the server is notified of sleeping changes.

All strings use a little-endian `u32` byte length followed by UTF-8 bytes.
