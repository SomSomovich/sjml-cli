# World biome lookup

The SDK exposes one read-only query for an already loaded position. With
`sjml` imported as a dependency, the example uses these imports:

```rust
use sjml::{world, BlockPos};

if let Some(handle) = world::get("minecraft:overworld")? {
    if let Some(biome_id) = world::biome_at(handle, BlockPos { x: 0, y: 64, z: 0 })? {
        // e.g. "minecraft:plains"
    }
}
```

The function is available to server-side modules. It requires `world.read` and
a valid module-scoped world handle. It runs on the owning logical server thread. The host checks build height and
`ServerLevel.hasChunkAt` before querying Minecraft's biome registry, so it
never loads a world or chunk. It returns `Ok(None)` for a position outside
build height or in an unloaded chunk; a loaded position returns a detached,
canonical lowercase `namespace:path` biome ID as a Rust `String`.

A loaded biome without a canonical registry key, or a malformed host registry
ID, is treated as a host error. The value is detached and does not retain a
Minecraft holder or world reference.

