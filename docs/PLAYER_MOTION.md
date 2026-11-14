# Server-side player motion and rotation

`player::motion` reads one live player session's velocity and rotation.
`player::set_velocity` replaces its three velocity components, and
`player::set_rotation` changes yaw and pitch. Velocity is measured in blocks
per tick.

Request `entity.read` to read motion and resolve player handles. Setting
velocity additionally requires `player.motion.write`; changing rotation
requires the separate `player.rotation.write`. The server administrator must
grant every requested capability:

```toml
[[modules]]
id = "main"
path = "modules/main.wasm"
side = "server"
permissions = ["entity.read", "player.motion.write", "player.rotation.write"]
```

```rust
use sjml::prelude::*;

fn boost_player(handle: PlayerHandle) -> SjmlResult<()> {
    let current = player::motion(handle)?;
    let direction = current.look_direction();
    host::log(
        LogLevel::Info,
        &format!("look direction: [{}, {}, {}]", direction[0], direction[1], direction[2]),
    )?;
    player::set_velocity(handle, player::PlayerVelocity::new(0.0, 0.8, 0.0)?)
}
```

```rust
use sjml::prelude::*;

fn face_player(handle: PlayerHandle, yaw: f32, pitch: f32) -> SjmlResult<()> {
    player::set_rotation(handle, yaw, pitch)
}
```

The Rust API is `player::motion(handle: PlayerHandle) ->
SjmlResult<PlayerMotion>` and `player::set_velocity(handle: PlayerHandle,
velocity: PlayerVelocity) -> SjmlResult<()>`. `PlayerMotion` contains
`velocity_x`, `velocity_y`, `velocity_z`, `yaw`, and `pitch`, in that order.
`PlayerMotion::look_direction()` derives a normalized `[f64; 3]` direction
vector locally from yaw and pitch; it makes no host call and does not add
fields to the motion record. It follows Minecraft's convention:
`x = -sin(yaw) * cos(pitch)`, `y = -sin(pitch)`, and
`z = cos(yaw) * cos(pitch)`, with angles in degrees. Yaw 0 and pitch 0 point
toward positive Z; positive yaw turns toward negative X and positive pitch
points downward.
`PlayerVelocity::new(x, y, z)` rejects non-finite components and components
outside -64..=64 blocks per tick. The host repeats these checks for direct ABI
callers and also rejects non-finite rotations from the player provider.
`player::set_rotation(handle: PlayerHandle, yaw: f32, pitch: f32) ->
SjmlResult<()>` requires `player.rotation.write` as well as `entity.read`.
Yaw must be finite and within ±36,000 degrees; pitch must be finite and within
-90..=90 degrees. The host wraps yaw to Minecraft's equivalent angle and
synchronizes the authoritative rotation to the player and tracking clients
without changing position or velocity.

All calls require a valid module-scoped `PlayerHandle` for an active login
session and run synchronously on the owning logical-server thread. Handles
survive respawn within the same login session, become stale when logout begins,
and are limited to 1,024 per module. Client contexts and static registration
fail closed with `PERMISSION_DENIED`. The velocity setter synchronizes motion
to the player and never loads a chunk or changes dimensions. The rotation
setter synchronizes the player's rotation and preserves position and velocity.

## ABI

All imports belong to `sjml_player_v1`, use `i32` arguments and return an
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

The `set_rotation` signature is
`(handle_ptr: i32, handle_len: i32, yaw_bits: i32, pitch_bits: i32) -> i32`.
The handle is exactly 24 bytes; `yaw_bits` and `pitch_bits` are the IEEE-754
bit patterns of two `f32` degree values. The host enforces the same finite and
range bounds as the SDK. The import is available only when both `entity.read`
and `player.rotation.write` are requested and granted. Static registration
and client contexts return `PERMISSION_DENIED`.
