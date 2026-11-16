# `player.item_smelted`

`player.item_smelted` reports that a `ServerPlayer` smelted an item on the
logical server. SJML listens to NeoForge 21.1.265's
`PlayerEvent.ItemSmeltedEvent`, which Minecraft posts after the smelting result
has been produced. The event is notification-only: it is not an
`ICancellableEvent` and exposes no result or setter, so a callback cannot change
or cancel the smelt.

The notification contains the player's UUID, the player's dimension, the
smelted item's canonical registry ID from `getSmelting()`, and its count. The
guest receives no `Player`, level, item stack, or other Minecraft object.

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

Each module may hold up to 256 event subscriptions. For one smelt, the host
invokes at most 64 subscribed callbacks. The complete payload is limited to
768 bytes, and each string is limited to 256 UTF-8 bytes.

## Rust handler

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("player.item_smelted", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.item_smelted")]
fn item_smelted(event: PlayerItemSmelted) -> SjmlResult<()> {
    host::log(
        LogLevel::Info,
        &format!(
            "{} smelted {} x{} in {}",
            event.player_id, event.item_id, event.count, event.dimension_id,
        ),
    )?;
    Ok(())
}
```

The compilable SDK example is
[`player_item_smelted.rs`](../sjml-cli/sjml-sdk/examples/gametest/player_item_smelted.rs).

## Schema v1

The callback export is `sjml_v1_event_player.item_smelted`. Its payload fields
use the stable little-endian encoding:

1. `u32(1)` schema version.
2. Length-prefixed UTF-8 canonical player UUID.
3. Length-prefixed UTF-8 canonical dimension resource location.
4. Length-prefixed UTF-8 canonical smelted item registry ID.
5. `i32` `count`: smelted item count in `1..=1_000_000`.

All strings use a little-endian `u32` byte length followed by UTF-8 bytes.
