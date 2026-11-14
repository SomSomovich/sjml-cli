# World chunk lifecycle events

The server event bus reports these observation-only notifications:

- `world.chunk_loaded` when NeoForge posts `ChunkEvent.Load` from `Chunk.onChunkLoad()`.
- `world.chunk_unloaded` when NeoForge posts `ChunkEvent.Unload` from `Chunk.onChunkUnload()`.

Both callbacks contain only a canonical `dimension_id`, `chunk_x`, and `chunk_z`. The host
copies those fields from the event and does not expose or retain the source `ChunkAccess`, level,
or server. The API never requests, loads, pins, or otherwise keeps a chunk resident. There is no
cancellation result.

## Lifecycle and execution

NeoForge 21.1.265 documents that `ChunkEvent.Load` can fire before the underlying chunk reaches
`FULL`, and warns that level interactions from the hook can deadlock chunk loading. The unload
event runs from the chunk's unload method while it is being removed. SJML therefore extracts the
dimension ID and coordinates immediately on the owning logical-server thread and passes only that
detached snapshot to authorized subscribers. Client-side events and callbacks arriving off the
owning server thread are ignored.

The regular `#[event]` callback runs synchronously on the server thread and can add work to the
chunk transition. Prefer `#[event_async]` for follow-up work: its wrapper validates the detached
payload, queues the guest future, and returns. The future still runs on SJML's logical-server
main-thread task lane, so it does not grant background-thread access to Minecraft state. Treat the
notification as an ID and coordinate snapshot; do not use it to start another chunk or level load.

Only modules with an active `world.chunk_loaded` or `world.chunk_unloaded` subscription and the
`events.subscribe` grant receive callbacks. The host invokes at most 64 callbacks for one source
event. Each schema-v1 payload is at most 272 bytes: a `u32` schema, a length-prefixed UTF-8
dimension ID (maximum 256 bytes), and two `i32` chunk coordinates. Dimension IDs must be canonical
resource locations. Chunk coordinates are limited to +/-1,875,000.

## Rust example

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("world.chunk_loaded", EventPriority::Normal)?;
    events::subscribe("world.chunk_unloaded", EventPriority::Normal)?;
    Ok(())
}

#[event_async(name = "world.chunk_loaded")]
async fn on_chunk_loaded(event: WorldChunkLifecycle) -> SjmlResult<()> {
    host::log(
        LogLevel::Info,
        &format!("loaded {} [{}, {}]", event.dimension_id, event.chunk_x, event.chunk_z),
    )?;
    Ok(())
}

#[event_async(name = "world.chunk_unloaded")]
async fn on_chunk_unloaded(event: WorldChunkLifecycle) -> SjmlResult<()> {
    host::log(
        LogLevel::Info,
        &format!("unloaded {} [{}, {}]", event.dimension_id, event.chunk_x, event.chunk_z),
    )?;
    Ok(())
}
```

The SDK source example is
[`world_chunk_lifecycle.rs`](../sjml-cli/sjml-sdk/examples/world_chunk_lifecycle.rs).

## Payload schema v1

The event exports are `sjml_v1_event_world.chunk_loaded` and
`sjml_v1_event_world.chunk_unloaded`. Both use the same little-endian payload:

1. `i32(1)` schema version.
2. Dimension ID: `u32` UTF-8 byte length followed by the canonical resource-location bytes.
3. `i32` chunk X coordinate.
4. `i32` chunk Z coordinate.

The event name identifies whether the chunk entered or left the level; no live game object crosses
the guest ABI.
