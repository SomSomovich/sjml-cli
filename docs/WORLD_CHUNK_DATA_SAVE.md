# World chunk data save event

The server event bus reports one observation-only notification:

- `world.chunk_data_save` when NeoForge posts `ChunkDataEvent.Save` from
  `ChunkMap#save(ChunkAccess)`, immediately before the chunk's serialized NBT is handed to
  storage.

The callback never carries NBT. The host copies only the owning dimension, the `chunk_x`/`chunk_z`
coordinates of `ChunkAccess#getPos()`, and `data_entry_count`, the number of top-level entries in
the `CompoundTag` returned by `ChunkDataEvent#getData()`. Neither the compound tag nor any level,
chunk, or server reference crosses the guest ABI. The API never requests, loads, pins, or otherwise
keeps a chunk resident; the chunk is already being saved. There is no cancellation result, and a
callback cannot delay or alter the write.

## Lifecycle and execution

NeoForge 21.1.265 fires `ChunkDataEvent.Save` only for the logical server. SJML extracts the
fields immediately on the owning logical-server thread and passes only that detached snapshot to
authorized subscribers. If the event's level is not a `ServerLevel`, the host reports nothing.
Client contexts and mismatched dedicated/integrated hosts never receive a snapshot, and a capture
that arrives off the owning server thread is rejected instead of being dispatched.

The regular `#[event]` callback runs synchronously on the server thread. `#[event_async]` is also
allowed: its wrapper validates the detached payload, queues the guest future, and returns. The
future still runs on SJML's logical-server main-thread task lane, so it does not grant
background-thread access to Minecraft state. The notification is a metadata snapshot; do not use
it to force another chunk or level load.

Only modules with an active `world.chunk_data_save` subscription and the `events.subscribe` grant
receive callbacks. No other capability is required and no cancellation capability is accepted. The
host invokes at most 64 callbacks for one source event. Each schema-v1 payload is at most 512
bytes.

## Payload schema v1

The event export is `sjml_v1_event_world.chunk_data_save`. Its little-endian payload is:

1. `i32(1)` schema version.
2. Dimension ID: `u32` UTF-8 byte length followed by the canonical resource-location bytes
   (maximum 256 bytes).
3. `i32` chunk X coordinate, limited to +/-4,000,000.
4. `i32` chunk Z coordinate, limited to +/-4,000,000.
5. `i32` `data_entry_count`, the number of top-level entries in the serialized compound tag
   (`0..=100,000,000`).

The count describes the tag's entry map as NeoForge exposes it at event time; it is not a byte
size, a compressed size, or a guarantee about the bytes eventually written to disk.

## Rust example

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("world.chunk_data_save", EventPriority::Normal)?;
    Ok(())
}

#[event_async(name = "world.chunk_data_save")]
async fn on_chunk_data_save(event: WorldChunkDataSave) -> SjmlResult<()> {
    host::log(
        LogLevel::Info,
        &format!(
            "saving {} chunk [{}, {}] with {} top-level NBT entries",
            event.dimension_id, event.chunk_x, event.chunk_z, event.data_entry_count
        ),
    )?;
    Ok(())
}
```

The SDK source example is
[`world_chunk_data_save.rs`](../sjml-sdk/examples/gametest/world_chunk_data_save.rs).
