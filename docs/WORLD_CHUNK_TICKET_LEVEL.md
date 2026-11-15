# World chunk ticket level event

The server event bus reports one observation-only notification:

- `world.chunk_ticket_level` when NeoForge posts `ChunkTicketLevelUpdatedEvent` from the server
  `ChunkMap` after a chunk's ticket level changes.

The callback contains only a canonical `dimension_id`, the `chunk_x`/`chunk_z` coordinates, the
`old_level`/`new_level` ticket values, and a `has_chunk_holder` flag. The host copies those fields
directly from the event and does not expose or retain the source `ServerLevel`, `ChunkHolder`, or
server. The API never requests, loads, pins, or otherwise keeps a chunk resident. There is no
cancellation result.

## Lifecycle and execution

NeoForge 21.1.265 fires this non-cancellable event only for the logical server, either from
`ChunkMap` ticket updates or when a ticket holder is released. Vanilla may report the same chunk
twice in one tick when a level decreases through an intermediate value; both transitions are
delivered. SJML extracts the fields immediately on the owning logical-server thread and passes only
that detached snapshot to authorized subscribers. Client contexts, mismatched
dedicated/integrated hosts, and callbacks arriving off the owning server thread never receive a
snapshot. The `has_chunk_holder` flag reflects whether NeoForge supplied a holder; it does not
imply the chunk is loaded or ticking.

The regular `#[event]` callback runs synchronously on the server thread. Prefer `#[event_async]`
for follow-up work: its wrapper validates the detached payload, queues the guest future, and
returns. The future still runs on SJML's logical-server main-thread task lane, so it does not grant
background-thread access to Minecraft state. Treat the notification as a detached scalar snapshot;
do not use it to force a chunk or level load.

Only modules with an active `world.chunk_ticket_level` subscription and the `events.subscribe`
grant receive callbacks. No cancellation capability is accepted. The host invokes at most 64
callbacks for one source event. Each schema-v1 payload is at most 512 bytes.

## Rust example

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("world.chunk_ticket_level", EventPriority::Normal)?;
    Ok(())
}

#[event_async(name = "world.chunk_ticket_level")]
async fn on_chunk_ticket_level(event: WorldChunkTicketLevel) -> SjmlResult<()> {
    host::log(
        LogLevel::Info,
        &format!(
            "{} [{}, {}] ticket {} -> {} (holder: {})",
            event.dimension_id,
            event.chunk_x,
            event.chunk_z,
            event.old_level,
            event.new_level,
            event.has_chunk_holder
        ),
    )?;
    Ok(())
}
```

The SDK source example is
[`world_chunk_ticket_level.rs`](../sjml-cli/sjml-sdk/examples/gametest/world_chunk_ticket_level.rs).

## Payload schema v1

The event export is `sjml_v1_event_world.chunk_ticket_level`. It uses a little-endian payload:

1. `i32(1)` schema version.
2. Dimension ID: `u32` UTF-8 byte length followed by the canonical resource-location bytes
   (maximum 256 bytes).
3. `i32` chunk X coordinate.
4. `i32` chunk Z coordinate.
5. `i32` old ticket level, in the range 0 to 64 inclusive.
6. `i32` new ticket level, in the range 0 to 64 inclusive.
7. `u8` chunk holder flag: `1` when the event carried a non-null holder, otherwise `0`.

Chunk coordinates are limited to +/-4,000,000. No live game object crosses the guest ABI.
