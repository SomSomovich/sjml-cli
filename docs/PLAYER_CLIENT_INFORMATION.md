# `player.client_information`

`player.client_information` reports that a `ServerPlayer` sent updated
server-synced client options. SJML listens to NeoForge 21.1.265's
`ClientInformationUpdatedEvent`, which Minecraft posts when the client settings
packet is applied to the logical server. The event is notification-only: it is
not an `ICancellableEvent` and exposes no result or setter, so a callback cannot
change or cancel the update.

The notification contains the player's UUID, the player's dimension, and the
updated `ClientInformation` values. Enum names are lowercase. The guest receives
no `Player`, level, `ClientInformation`, or other Minecraft object.

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

Each module may hold up to 256 event subscriptions. For one update, the host
invokes at most 64 subscribed callbacks. The complete payload is limited to
1 KiB, and each string is limited to 256 UTF-8 bytes.

## Rust handler

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("player.client_information", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.client_information")]
fn client_information(event: PlayerClientInformation) -> SjmlResult<()> {
    host::log(
        LogLevel::Info,
        &format!(
            "{} in {} uses {} (view_distance={}, chat={}, main_hand={})",
            event.player_id,
            event.dimension_id,
            event.language,
            event.view_distance,
            event.chat_visibility,
            event.main_hand,
        ),
    )?;
    Ok(())
}
```

The compilable SDK example is
[`player_client_information.rs`](../sjml-cli/sjml-sdk/examples/gametest/player_client_information.rs).

## Schema v1

The callback export is `sjml_v1_event_player.client_information`. Its payload
fields use the stable little-endian encoding:

1. `u32(1)` schema version.
2. Length-prefixed UTF-8 canonical player UUID.
3. Length-prefixed UTF-8 canonical dimension resource location.
4. Length-prefixed UTF-8 language code, 1 to 64 bytes, no control characters.
5. `i32` `view_distance`, between 0 and 32.
6. Length-prefixed UTF-8 lowercase `chat_visibility` enum name.
7. `u8` `chat_colors`: `1` when chat colors are enabled.
8. `i32` `model_customisation`, between 0 and 255.
9. Length-prefixed UTF-8 lowercase `main_hand` enum name.
10. `u8` `text_filtering_enabled`: `1` when filtered text is requested.
11. `u8` `allows_listing`: `1` when the player may appear in the server list.

All strings use a little-endian `u32` byte length followed by UTF-8 bytes.
