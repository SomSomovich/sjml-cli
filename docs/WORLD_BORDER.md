# World border API

`world::border` reads the border settings of a loaded dimension. The returned
`WorldBorderSnapshot` contains the center, current diameter, transition target
and remaining transition time, warning lead time and distance, and damage
buffer and per-block damage amount. Transition time is milliseconds remaining;
a stationary border has a zero remaining time and equal current and target sizes.

Reads require `world.read`. Setters require `world.write` as well as a loaded
`WorldHandle` on the owning logical-server thread. Client and static-registration
calls are denied. These operations do not inspect or load chunks.

```toml
[[modules]]
id = "main"
path = "modules/main.wasm"
side = "server"
permissions = ["world.read", "world.write"]
```

```rust
use sjml::world;

fn make_border(handle: sjml::WorldHandle) -> sjml::SjmlResult<()> {
    let before = world::border(handle)?;
    world::set_border_center(handle, before.center_x, before.center_z)?;
    world::set_border_size(handle, 2_000.0, 60_000)?;
    world::set_border_warnings(handle, 90, 32)?;
    world::set_border_damage(handle, 8.0, 0.4)
}
```

The host accepts center coordinates from -29,999,984 to 29,999,984, diameter
from 1 to 59,999,968 blocks, transition time from 0 to 604,800,000 milliseconds,
warning time from 0 to 86,400 seconds, warning distance from 0 to 59,999,968
blocks, damage buffer from 0 to 1,024 blocks, and damage per block from 0 to 100.
All floating-point inputs must be finite. A zero transition sets the size
immediately; a positive value starts a server-controlled size interpolation.

## ABI

All imports are in `sjml_world_v1` and return an `i32` status. `border` has
signature `(handle_ptr, handle_len, output_ptr, output_capacity) -> i32`. It writes
an exact 68-byte schema-v1 snapshot: little-endian `u32 schema=1`, `f64 center_x`,
`f64 center_z`, `f64 current_diameter`, `f64 target_diameter`, `u64 remaining_lerp_millis`,
`i32 warning_time_seconds`, `i32 warning_distance_blocks`, `f64 damage_buffer_blocks`,
and `f64 damage_per_block`. The handle is exactly 24 bytes and output capacity
must be at least 68 bytes.

Each setter has signature
`(handle_ptr, handle_len, values_ptr, values_len) -> i32`; the handle is 24 bytes.
`values_ptr` points to an exact schema-v1 encoding:

| Import | Bytes | Fields after `u32 schema=1` |
| --- | ---: | --- |
| `set_border_center` | 20 | `f64 center_x`, `f64 center_z` |
| `set_border_size` | 20 | `f64 diameter`, `u64 transition_millis` |
| `set_border_warnings` | 12 | `u32 warning_time_seconds`, `u32 warning_distance_blocks` |
| `set_border_damage` | 20 | `f64 damage_buffer_blocks`, `f64 damage_per_block` |

Malformed lengths or out-of-range values return `INVALID_ARGUMENT`; unknown
schemas return `INCOMPATIBLE_ABI_VERSION`. Static registration and client
contexts return `PERMISSION_DENIED` for mutation.
