# Player vitals mutation API

The Rust SDK can set a player's current health, food and saturation, and experience. These operations run synchronously on the owning logical server and act on the current live login session. They do not expose Minecraft objects.

```rust
use sjml::{player, PlayerHandle, SjmlResult};

fn restore_player(player: PlayerHandle) -> SjmlResult<()> {
    player::set_health(player, 20.0)?;
    player::set_food(player, 20, 5.0)?;
    player::set_experience(player, 10, 0.5)?;
    Ok(())
}
```

The module must declare and receive both `entity.read` and the separate `player.vitals.write` capability:

```toml
[[modules]]
id = "main"
path = "modules/main.wasm"
side = "server"
permissions = ["entity.read", "player.vitals.write"]
```

The host validates each complete request before changing state:

| Function | Accepted values |
| --- | --- |
| `set_health(handle, health)` | Finite `0..=1,000,000`, and no greater than the player's current maximum health. Setting zero triggers normal Minecraft death handling. |
| `set_food(handle, food_level, saturation)` | Food level `0..=20`; finite saturation from zero through the requested food level. Both values are applied together. |
| `set_experience(handle, level, progress)` | Level `0..=20,000`; finite progress in `[0, 1)`. Minecraft stores whole XP points, so progress is rounded to the nearest point for that level and capped below the next level. |

The host checks `entity.read`, the write grant, the logical-server context, the owning server thread, and the current player handle on each call. Respawn keeps a handle attached to its login session; logout, stale handles, client execution, static registration, and a non-owning thread cannot mutate the player. Client-only modules do not receive the write imports. A module with `side = "both"` can use them only from its server instance.

## WebAssembly imports

All imports are in `sjml_player_v1` and return an ABI status code. Each takes the standard 24-byte ABI v1 player handle as `handle_ptr, handle_len`:

- `set_health(handle_ptr, handle_len, health_bits)`: `health_bits` is the IEEE-754 `f32` bit pattern carried in an `i32`.
- `set_food(handle_ptr, handle_len, food_level, saturation_bits)`: `food_level` is a `u32`; `saturation_bits` carries the IEEE-754 `f32` bit pattern in an `i32`.
- `set_experience(handle_ptr, handle_len, level, progress_bits)`: `level` is a `u32`; `progress_bits` carries the IEEE-754 `f32` bit pattern in an `i32`.

Malformed handles and values outside these bounds return `INVALID_ARGUMENT`. Missing permissions, client/static contexts, and calls away from the owning server thread return `PERMISSION_DENIED`. The host rejects an inactive login session without changing player state.
