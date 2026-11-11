# Server-side player motion

`player::motion` reads one live player session's velocity and rotation.
`player::set_velocity` replaces its three velocity components. Rotation is
read-only through this API. Velocity is measured in blocks per tick.

Request `entity.read` to read motion and resolve player handles. Setting
velocity additionally requires `player.motion.write`; the server administrator
must grant every requested capability:

```toml
[[modules]]
id = "main"
path = "modules/main.wasm"
side = "server"
permissions = ["entity.read", "player.motion.write"]
```

```rust
use sjml::prelude::*;

fn boost_player(handle: PlayerHandle) -> SjmlResult<()> {
    let current = player::motion(handle)?;
    host::log(LogLevel::Info, &format!("yaw={}, pitch={}", current.yaw, current.pitch))?;
    player::set_velocity(handle, player::PlayerVelocity::new(0.0, 0.8, 0.0)?)
}
```

The Rust API is `player::motion(handle: PlayerHandle) ->
SjmlResult<PlayerMotion>` and `player::set_velocity(handle: PlayerHandle,
velocity: PlayerVelocity) -> SjmlResult<()>`. `PlayerMotion` contains
`velocity_x`, `velocity_y`, `velocity_z`, `yaw`, and `pitch`, in that order.
`PlayerVelocity::new(x, y, z)` rejects non-finite components and components
outside -64..=64 blocks per tick. The host repeats these checks for direct ABI
callers and also rejects non-finite rotations from the player provider.

Both calls require a valid module-scoped `PlayerHandle` for an active login
session and run synchronously on the owning logical-server thread. Handles
survive respawn within the same login session, become stale when logout begins,
and are limited to 1,024 per module. Client contexts and static registration
fail closed with `PERMISSION_DENIED`. The setter marks the motion for server
tracking and synchronizes the player owner; it never loads a chunk or changes
dimensions.

## ABI

Both imports belong to `sjml_player_v1`, use `i32` arguments and return an
`i32` status (zero for success, otherwise a stable ABI error). The `motion`
signature is
`(handle_ptr: i32, handle_len: i32, output_ptr: i32, output_capacity: i32) -> i32`.
It requires the exact 24-byte player handle and writes a 36-byte schema-v1
record: little-endian `u32 schema=1`, `f64 velocity_x`, `f64 velocity_y`,
`f64 velocity_z`, `f32 yaw`, and `f32 pitch`. The output capacity must be at
least 36 bytes.

The `set_velocity` signature is
`(handle_ptr: i32, handle_len: i32, velocity_ptr: i32, velocity_len: i32) -> i32`.
The handle is exactly 24 bytes. `velocity_ptr` addresses an exact 28-byte
schema-v1 record: little-endian `u32 schema=1`, `f64 x`, `f64 y`, and `f64 z`.
All three components must be finite and within ±64. Unknown schemas return
`INCOMPATIBLE_ABI_VERSION`; malformed lengths or values return
`INVALID_ARGUMENT`. The import is available only when both `entity.read` and
`player.motion.write` are requested and granted. Static registration and
client contexts return `PERMISSION_DENIED`.
