# World heightmaps and light

The SDK exposes read-only queries for positions whose chunks are already loaded:

```rust
use sjml::{world, BlockPos, HeightmapType};

let surface_y = world::heightmap(handle, HeightmapType::WorldSurface, 0, 0)?;
let light = world::light_at(handle, BlockPos { x: 0, y: 64, z: 0 })?;
if let Some(levels) = light {
    // Each level is in 0..=15.
    let sky = levels.sky;
    let block = levels.block;
}
```

Both calls require a valid module-scoped world handle, `world.read`, and the owning logical-server
thread. They are unavailable to client-side modules. Coordinates x and z must be within
`-30_000_000..=30_000_000`; values beyond that bound return `InvalidArgument`.

`heightmap` accepts four runtime maps. Its return value is the highest Y included by the selected
map; an entirely empty column returns one below the world's minimum build height.

| Rust variant | Value | Vanilla meaning |
| --- | ---: | --- |
| `HeightmapType::WorldSurface` | 0 | Highest non-air block. |
| `HeightmapType::OceanFloor` | 1 | Highest motion-blocking block, excluding fluids. |
| `HeightmapType::MotionBlocking` | 2 | Highest motion-blocking block or fluid. |
| `HeightmapType::MotionBlockingNoLeaves` | 3 | Motion-blocking height while disregarding leaves. |

World-generation-only heightmaps are not exposed. If the chunk is unloaded or the requested map
has not been primed, the call returns `Ok(None)`.

`light_at` returns a detached `LightSnapshot` with `sky` and `block` levels, each from 0 through
15. It returns `Ok(None)` outside build height or if the position's chunk is not loaded. A
dimension without sky lighting can report a sky level of zero.

The host uses a non-loading lookup for each chunk before reading its heightmap or light engine.
Neither call loads a world or chunk. The stable imports are
`sjml_world_v1.heightmap` and `sjml_world_v1.light_at`; their optional results use the versioned
world-v1 ABI.
