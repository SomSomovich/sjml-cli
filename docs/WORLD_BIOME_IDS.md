# Enumerating registered biome IDs

Use `world::biome_ids` to list the canonical biome registry keys available to one loaded world:

```rust
use sjml::{world, SjmlResult, WorldHandle};

fn log_biomes(world_handle: WorldHandle) -> SjmlResult<()> {
    for id in world::biome_ids(world_handle)? {
        sjml::host::log(sjml::LogLevel::Info, &id)?;
    }
    Ok(())
}
```

The result includes vanilla and modded biome IDs, sorted lexicographically by canonical
`namespace:path` key. Enumeration reads the loaded `ServerLevel`'s biome registry directly and
never requests a chunk or dimension. The module must request and receive `world.read`, use its own
live world handle, and execute on the owning logical-server thread.

The result is complete or an error. A registry with more than 4,000 entries is rejected; each ID is
limited to 256 UTF-8 bytes. The full response limit is 1,040,008 bytes, below the 1 MiB ABI payload
limit.

## ABI v1

`sjml_world_v1.biome_ids(query_ptr, query_len, output_ptr, output_capacity) -> i32` takes a 28-byte
request with schema `u32` followed by a 24-byte world handle. The response starts with schema and
count `u32` fields, then each canonical ID as a `u32` UTF-8 byte length and bytes. The host writes
nothing unless the entire response fits the output buffer.
