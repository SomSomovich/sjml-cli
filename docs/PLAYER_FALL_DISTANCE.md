# Player fall-distance reset

The player API exposes a guarded operation to reset fall-distance tracking to vanilla's zero state:

```rust
player::reset_fall_distance(player_handle)?;
let movement = player::movement_state(player_handle)?;
assert_eq!(movement.fall_distance, 0.0);
```

The module must declare and receive both `entity.read` and the separate `player.fall_distance.write` capability. The operation is available only to logical-server modules on the owning server thread, and the host resolves the current live player login session for every call. Client contexts, stale handles, and static registration are denied.

This API accepts no caller-provided fall-distance value. It delegates to Minecraft 1.21.1's `Entity.resetFallDistance()`, which resets the player's fall-distance tracking to zero. Other movement flags are not modified. The operation does not load a world or locate entities.

```toml
[[modules]]
id = "main"
path = "modules/main.wasm"
side = "server"
permissions = ["entity.read", "player.fall_distance.write"]
```

The SDK function is `player::reset_fall_distance(handle) -> SjmlResult<()>`. Its `sjml_player_v1` import is `reset_fall_distance(handle_ptr, handle_len)` and returns the standard SJML ABI status code.
