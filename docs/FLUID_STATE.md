# Loaded fluid state

Server modules can read a detached snapshot for one block position:

```rust
use sjml::prelude::*;

fn fluid_at(world: WorldHandle, x: i32, y: i32, z: i32)
    -> SjmlResult<Option<FluidStateSnapshot>>
{
    world::fluid_at(world, BlockPos { x, y, z })
}
```

The module must request `world.read`, receive that grant, and run in the logical
server context. The host reads only the addressed position. `None` means the
position is outside build height or its chunk is not already loaded; the call
never loads a chunk. A loaded dry block returns `minecraft:empty` with amount
zero, `source = false`, and height zero.

`FluidStateSnapshot` contains the canonical fluid registry ID, vanilla amount
in `0..=8`, source flag, and intrinsic `FluidState::getOwnHeight()` in
`0..=1` blocks. Flowing fluids can use a different registry ID, such as
`minecraft:flowing_water`. The height is read from that state alone and does
not inspect neighboring blocks, so it is not the final rendered/collision
surface height.

The ABI uses a strict schema-v1 position query and bounded optional response.
Focused coverage is in `WorldFluidStateCodecTest`, `WorldFluidStateSnapshotTest`,
`WorldHandleRegistryTest`, and `WorldReadImportsTest`. The server GameTest and a
visible fluid interaction still need a real Minecraft run.
