# Non-player entity velocity

ABI 1.36.0 and SDK 0.44.0 add bounded velocity reads and writes for loaded
living non-player entities. Calls are synchronous on the owning logical server
thread. The host resolves the opaque entity handle again on every call; this
API never loads chunks or exposes a Minecraft entity object.

Request `entity.read` to read velocity. Mutation additionally requires the
separate `entity.velocity.write` capability, and both permissions must be
declared by the module and granted by the administrator:

```toml
[[modules]]
id = "main"
path = "modules/main.wasm"
side = "server"
permissions = ["entity.read", "entity.velocity.write"]
```

```rust
use sjml::prelude::*;

fn launch(handle: EntityHandle) -> SjmlResult<()> {
    let current = entity::velocity(handle)?;
    let upward = EntityVelocity::new(current.x, 0.8, current.z)?;
    entity::set_velocity(handle, upward)
}
```

`EntityVelocity` is measured in blocks per tick. Each component must be finite
and within `[-64, 64]`. The SDK validates locally, and the host repeats the
validation before mutation. Velocity is available only for loaded living
non-player entities; player handles belong to the separate player API, and
non-living handles are rejected. Stale handles, client contexts, wrong-thread
calls, and unloaded entities are rejected without changing velocity.

The versioned schema is exactly 28 bytes: little-endian `u32(1)`, then little-
endian `f64` values `x`, `y`, and `z`. The imports are
`sjml_entity_v1.velocity(handle_ptr, handle_len, output_ptr, output_capacity)`
and `sjml_entity_v1.set_velocity(handle_ptr, handle_len, velocity_ptr,
velocity_len)`; both return the normal signed ABI status in an `i32`.
Handles use the existing 24-byte schema. Reads preflight the complete output
before querying the entity and may use the bounded batch dispatcher; writes
require an exact schema-v1 input and are not batchable. Invalid schemas or
lengths return `INVALID_ARGUMENT`, short output buffers return
`PAYLOAD_TOO_LARGE`, and static-registration calls are linked but denied.

Mutation replaces all three server-side delta-movement components and marks
the entity motion dirty so the normal tracking path sends the update to clients.
Bounds apply independently to each component; the vector is not normalized.
