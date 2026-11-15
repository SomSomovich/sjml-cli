# World chunk watch events

The server event bus reports these observation-only notifications:

- `world.chunk_watch` when NeoForge posts `ChunkWatchEvent.Watch` after a chunk is queued for
  sending to a `ServerPlayer`.
- `world.chunk_unwatch` when NeoForge posts `ChunkWatchEvent.UnWatch` after a chunk leaves that
  player's watched set.

Both callbacks contain only a canonical `player_id` UUID, a canonical `dimension_id`, and the
`chunk_x`/`chunk_z` coordinates. The host copies those fields directly from the event and does not
expose or retain the source `ServerPlayer`, `LevelChunk`, level, or server. The API never requests,
loads, pins, or otherwise keeps a chunk resident. There is no cancellation result.

## Lifecycle and execution

NeoForge 21.1.265 fires both events only for the logical server. `ChunkWatchEvent.Watch` runs after
Minecraft decides to queue a chunk for the player, while `UnWatch` runs when the chunk is dropped
from that player's watch set; the chunk named by `UnWatch` may never have reached the client on a
slow connection. SJML extracts the fields immediately on the owning logical-server thread and
passes only that detached snapshot to authorized subscribers. Client contexts, mismatched
dedicated/integrated hosts, and callbacks arriving off the owning server thread never receive a
snapshot.

The regular `#[event]` callback runs synchronously on the server thread. Prefer `#[event_async]`
for follow-up work: its wrapper validates the detached payload, queues the guest future, and
returns. The future still runs on SJML's logical-server main-thread task lane, so it does not grant
background-thread access to Minecraft state. The notification is an identity and coordinate
snapshot; do not use it to force another chunk or level load.

Only modules with an active `world.chunk_watch` or `world.chunk_unwatch` subscription and the
`events.subscribe` grant receive callbacks. No cancellation capability is accepted. The host
invokes at most 64 callbacks for one source event. Each schema-v1 payload is at most 1024 bytes: an
`i32` schema version, a length-prefixed canonical player UUID (maximum 256 bytes), a
length-prefixed canonical dimension resource location (maximum 256 bytes), and two `i32` chunk
coordinates. Chunk coordinates are limited to +/-4,000,000.

## Rust example

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("world.chunk_watch", EventPriority::Normal)?;
    events::subscribe("world.chunk_unwatch", EventPriority::Normal)?;
    Ok(())
}

#[event_async(name = "world.chunk_watch")]
async fn on_chunk_watch(event: WorldChunkWatched) -> SjmlResult<()> {
    host::log(
        LogLevel::Info,
        &format!(
            "player {} watches {} [{}, {}]",
            event.player_id, event.dimension_id, event.chunk_x, event.chunk_z
        ),
    )?;
    Ok(())
}

#[event_async(name = "world.chunk_unwatch")]
async fn on_chunk_unwatch(event: WorldChunkUnwatched) -> SjmlResult<()> {
    host::log(
        LogLevel::Info,
        &format!(
            "player {} stopped watching {} [{}, {}]",
            event.player_id, event.dimension_id, event.chunk_x, event.chunk_z
        ),
    )?;
    Ok(())
}
```

The SDK source example is
[`world_chunk_watch.rs`](../sjml-cli/sjml-sdk/examples/gametest/world_chunk_watch.rs).

## Payload schema v1

The event exports are `sjml_v1_event_world.chunk_watch` and
`sjml_v1_event_world.chunk_unwatch`. Both use the same little-endian payload:

1. `i32(1)` schema version.
2. Player UUID: `u32` UTF-8 byte length followed by the canonical lowercase UUID bytes.
3. Dimension ID: `u32` UTF-8 byte length followed by the canonical resource-location bytes.
4. `i32` chunk X coordinate.
5. `i32` chunk Z coordinate.

The event name identifies whether the player started or stopped watching the chunk; no live game
object crosses the guest ABI.
