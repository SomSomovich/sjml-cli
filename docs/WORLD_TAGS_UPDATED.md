# `world.tags_updated`

`world.tags_updated` reports that registry tags were rebound after the logical
server reloaded tags from datapacks. SJML listens to NeoForge 21.1.265's
`TagsUpdatedEvent`, which is posted after the server's `TagManager` has applied
the new tag data. The event is notification-only: it is not an
`ICancellableEvent` and exposes no result or setter, so a callback cannot change
or cancel the reload.

NeoForge can also post the underlying hook on clients. SJML dispatches this
notification only for logical-server contexts: the host requires a current
server and verifies that the calling thread is the server thread before
dispatching. The event has no level, so the guest receives only the detached
cause and connection flags.

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

Each module may hold up to 256 event subscriptions. For one tag update, the
host invokes at most 64 subscribed callbacks. The complete payload is limited
to 256 bytes, and the cause string is limited to 64 UTF-8 bytes.

## Rust handler

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("world.tags_updated", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "world.tags_updated")]
fn tags_updated(event: WorldTagsUpdated) -> SjmlResult<()> {
    host::log(
        LogLevel::Info,
        &format!(
            "tags updated (cause={}, from_client_packet={}, integrated_server_connection={})",
            event.cause, event.from_client_packet, event.integrated_server_connection,
        ),
    )?;
    Ok(())
}
```

The compilable SDK example is
[`world_tags_updated.rs`](../sjml-cli/sjml-sdk/examples/gametest/world_tags_updated.rs).

## Schema v1

The callback export is `sjml_v1_event_world.tags_updated`. Its payload fields
use the stable little-endian encoding:

1. `u32(1)` schema version.
2. Length-prefixed UTF-8 cause: exactly `server_data_load` or
   `client_packet_received`.
3. `u8` `from_client_packet`: `1` when the cause is `client_packet_received`.
4. `u8` `integrated_server_connection`: `1` when a client packet arrived from an
   integrated server connection.

All strings use a little-endian `u32` byte length followed by UTF-8 bytes.
