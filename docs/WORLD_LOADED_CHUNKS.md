# Loaded chunk enumeration

Use `world::loaded_chunks` to list chunks that are already present around a block position:

```rust
use sjml::{world, BlockPos, SjmlResult, WorldHandle};

fn nearby_loaded_chunks(world_handle: WorldHandle) -> SjmlResult<()> {
    let chunks = world::loaded_chunks(
        world_handle,
        BlockPos { x: 120, y: 64, z: -48 },
        2,
    )?;
    for chunk in chunks {
        sjml::host::log(
            sjml::LogLevel::Info,
            &format!("loaded chunk {}, {}", chunk.x, chunk.z),
        )?;
    }
    Ok(())
}
```

The center uses its X/Z coordinates; its Y coordinate is ignored. The host converts X/Z to
Minecraft chunk coordinates using floor division, then examines the square from `-radius` through
`+radius` on each axis. The radius is measured in chunks and is limited to 16. Results are sorted by
chunk X and then Z. The call returns the complete result, or `PayloadTooLarge` if more than 512
chunks are loaded inside the square; it never truncates the result.

The module must request and receive `world.read`, pass a live world handle owned by that module,
and run in the owning logical-server context and thread. The host checks residency with
`ServerChunkCache.hasChunk` and never requests, generates, or loads a chunk or dimension. Client
modules and static registration cannot enumerate server chunks.

## ABI v1

`sjml_world_v1.loaded_chunks(query_ptr, query_len, output_ptr, output_capacity) -> i32` uses the
stable schema-v1 records. The 48-byte request contains a schema `u32`, a 24-byte world handle, a
versioned `BlockPos`, and a radius `u32`. The response starts with schema and count `u32` values,
then up to 512 versioned chunk positions, each `{x: i32, z: i32}`. Its maximum encoded size is
6,152 bytes. The host writes no response bytes unless the entire result fits the caller's output
buffer.
