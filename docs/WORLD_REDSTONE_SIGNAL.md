# Neighboring redstone signal

`world::best_neighbor_signal` reads the maximum vanilla signal next to a block position:

```rust
use sjml::{world, BlockPos, ErrorCode, SjmlError, SjmlResult};

fn read_signal(x: i32, y: i32, z: i32) -> SjmlResult<Option<u8>> {
    let overworld = world::get("minecraft:overworld")?.ok_or_else(|| {
        SjmlError::new(ErrorCode::InvalidArgument, "the Overworld is not loaded")
    })?;
    world::best_neighbor_signal(overworld, BlockPos { x, y, z })
}
```

The result matches Minecraft 1.21.1's `SignalGetter::getBestNeighborSignal`: it checks the six
blocks directly adjacent to the requested position using vanilla's signal and direct-power rules.
The returned value is always from `0..=15`; zero is a present reading and means that none of those
neighbors emits a signal. This reads vanilla's local signal value, not a full redstone circuit
simulation.

Reads require a live module-scoped world handle, the requested and granted `world.read` capability,
and the owning logical-server thread. Client modules and static registration cannot read it. X/Z
outside `-30_000_000..=30_000_000` are rejected. The requested position must be inside the world's
build-height range.

Before it asks Minecraft for the signal, the host checks all positions the vanilla query could
inspect: the six adjacent blocks and the six direct-signal neighbors around each of them. Every
such position that is within build height must fall inside Minecraft's horizontal world bounds and
already belong to a full loaded chunk. If the target is outside build height or any required chunk
is unavailable, the call returns `Ok(None)`. The host makes these checks with
`ServerChunkCache.hasChunk`; it does not request, generate, or load chunks. Because the preflight
covers all positions vanilla could inspect, it can return `None` for an unloaded outer neighbor
even when a subset of the loaded neighbors would currently provide a signal.

## ABI v1

The import is `sjml_world_v1.best_neighbor_signal(handle_ptr, handle_len, x, y, z, output_ptr,
output_capacity) -> i32`. It requires the same `world.read` permission as other world reads. The
output is a fixed five-byte optional `u32`: a presence byte followed by the little-endian signal
value (zero when absent). The Rust wrapper validates that a present value is at most 15 and exposes
`SjmlResult<Option<u8>>`.
