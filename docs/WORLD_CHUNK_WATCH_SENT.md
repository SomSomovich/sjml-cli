# World chunk watch sent event

The server event bus reports one observation-only notification:

- `world.chunk_watch_sent` when NeoForge posts `ChunkWatchEvent.Sent` after a chunk has actually
  been transmitted to a `ServerPlayer`'s client.

The callback contains only a canonical `dimension_id`, a canonical `player_id` UUID, and the
`chunk_x`/`chunk_z` coordinates. The host copies those fields directly from the event and does not
expose or retain the source `ServerPlayer`, `LevelChunk`, level, or server. The API never requests,
loads, pins, or otherwise keeps a chunk resident. There is no cancellation result.

## Lifecycle and execution

NeoForge 21.1.265 fires this non-cancellable event only for the logical server, from
`PlayerChunkSender#sendNextChunks`, after the chunk packet reaches the player's client. It differs
from `world.chunk_watch`, which fires when the chunk is merely queued for sending. Because the
client now knows the chunk, follow-up chunk-related data may safely be sent from this callback; the
notification itself stays observational and must not force another chunk or level load.

SJML extracts the fields immediately on the owning logical-server thread and passes only that
detached snapshot to authorized subscribers. Client contexts, mismatched dedicated/integrated
hosts, and callbacks arriving off the owning server thread never receive a snapshot. Only the
`events.subscribe` grant is required; no cancellation capability is accepted.

The regular `#[event]` callback runs synchronously on the server thread. Asynchronous observation
is allowed: `#[event_async]` validates the detached payload, queues the guest future, and returns.
The future still runs on SJML's logical-server main-thread task lane, so it does not grant
background-thread access to Minecraft state.

The host invokes at most 64 callbacks for one source event. Each schema-v1 payload is at most 512
bytes: an `i32` schema version, a length-prefixed canonical dimension resource location (maximum
256 bytes), a length-prefixed canonical player UUID (maximum 256 bytes), and two `i32` chunk
coordinates. Chunk coordinates are limited to +/-4,000,000.

## Rust example

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("world.chunk_watch_sent", EventPriority::Normal)?;
    Ok(())
}

#[event_async(name = "world.chunk_watch_sent")]
async fn on_chunk_watch_sent(event: WorldChunkWatchSent) -> SjmlResult<()> {
    host::log(
        LogLevel::Info,
        &format!(
            "player {} received {} [{}, {}]",
            event.player_id, event.dimension_id, event.chunk_x, event.chunk_z
        ),
    )?;
    Ok(())
}
```

The SDK source example is
[`world_chunk_watch_sent.rs`](../sjml-cli/sjml-sdk/examples/gametest/world_chunk_watch_sent.rs).

## Payload schema v1

The event export is `sjml_v1_event_world.chunk_watch_sent`. It uses a little-endian payload:

1. `i32(1)` schema version.
2. Dimension ID: `u32` UTF-8 byte length followed by the canonical resource-location bytes
   (maximum 256 bytes).
3. Player UUID: `u32` UTF-8 byte length followed by the canonical lowercase UUID bytes
   (maximum 256 bytes).
4. `i32` chunk X coordinate.
5. `i32` chunk Z coordinate.

No live game object crosses the guest ABI.
