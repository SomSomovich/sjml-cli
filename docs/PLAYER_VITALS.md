# Player vitals

The Rust SDK exposes `player::vitals(handle)` as a read-only snapshot of a live
player's health, food, and experience. The module must request and receive
`entity.read`, and the call must run in the owning logical-server context.
Client modules have no server player handles.

```rust
let vital_data = sjml::player::vitals(player_handle)?;
if vital_data.food_level < 6 || vital_data.health < 8.0 {
    // Decide what the mod should do; this snapshot cannot mutate the player.
}
```

`PlayerVitals` is a detached schema-v1 record containing:

| Field | Meaning | Bound |
| --- | --- | --- |
| `health` | Current health | finite, 0–1,000,000 |
| `max_health` | Current maximum health | finite, greater than 0 and at most 1,000,000 |
| `food_level` | Hunger/food level | 0–20 |
| `saturation` | Food saturation | finite, 0–20 |
| `experience_level` | Current XP level | 0–1,000,000 |
| `experience_progress` | Progress toward the next level | finite, 0–1 |

Each call revalidates the opaque handle against its owner, capability, server
thread, and current login session. A handle follows the player across respawn;
the host reads the replacement server-player object. Logout and runtime shutdown
invalidate the handle. If a modded player supplies non-finite or out-of-range
values, the operation returns `INVALID_ARGUMENT` rather than forwarding invalid
floats through the ABI.

The WebAssembly import is `sjml_player_v1.vitals(handle_ptr, handle_len,
output_ptr, output_capacity) -> status`. Its payload is a little-endian
schema-version `i32` followed by `health: f32`, `max_health: f32`,
`food_level: u32`, `saturation: f32`, `experience_level: u32`, and
`experience_progress: f32`. The exact output size is 28 bytes. Static
registration resolves the import but denies runtime player access.
