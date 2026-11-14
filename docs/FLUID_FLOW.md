# Fluid flow

`world::fluid_flow_at(world, position)` returns a detached `FluidFlowSnapshot` containing the
Minecraft 1.21.1 flow vector for one loaded, non-empty fluid cell.

```rust
let Some(flow) = world::fluid_flow_at(world, position)? else {
    // No fluid at the target, the target is unavailable, or flow calculation needed an unloaded
    // neighbor.
    return Ok(());
}

// The components are finite. Vanilla returns a normalized direction or (0, 0, 0).
let eastward = flow.x > 0.0;
```

The API requires `world.read` and a logical-server context. It returns `None` when the position is
outside build height, its chunk is unloaded, the target fluid state is empty, or flow calculation
reads a neighboring position from an unloaded chunk. It never loads chunks. A present zero vector
means the loaded fluid has no net flow; `None` means no vector was available.

Minecraft 1.21.1 exposes `FluidState.getFlow(BlockGetter, BlockPos)`. Vanilla flowing fluids inspect
the four horizontal neighbors, and may inspect the fluid below a non-blocking empty neighbor. For a
falling state, they also inspect neighboring blocks and fluids at the same height and above. SJML
passes a `BlockGetter` that only accesses already-loaded chunks and returns `None` if any lookup
would otherwise see unloaded chunk data. This also bounds custom fluid implementations that query
additional positions through the supplied getter.

The direction is a detached finite vector with components in [-1, 1] and normalized length at most
1.000001 to allow floating-point rounding. No Minecraft object or handle crosses the ABI.
