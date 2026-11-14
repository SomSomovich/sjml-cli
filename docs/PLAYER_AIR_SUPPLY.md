# Player air supply

The player API can read a live server player's current and maximum air supply and set the current value:

```rust
let original = player::air_supply(player_handle)?;
if original.current >= 0 {
    player::set_air_supply(player_handle, original.max)?;
    player::set_air_supply(player_handle, original.current as u32)?;
}
```

`air_supply` returns a detached `PlayerAirSupply { current, max }` snapshot. `current` may be as low as `-20` during vanilla's drowning damage phase. `max` comes from the current player entity, so it reflects entity-specific changes. The snapshot contains no Minecraft objects.

Reading requires `entity.read`. Writing requires both `entity.read` and the separate `player.air.write` capability, and runs only on the owning logical server thread against the current online login session. Client modules and static registration cannot read or write a live player's air value. The host rechecks the session and current maximum for every write; a requested value must be in `0..=maxAirSupply`, so a stale snapshot cannot write above a newly reduced maximum.

```toml
[[modules]]
id = "main"
path = "modules/main.wasm"
side = "server"
permissions = ["entity.read", "player.air.write"]
```

The SDK functions are `player::air_supply(handle) -> SjmlResult<PlayerAirSupply>` and `player::set_air_supply(handle, air_supply: u32) -> SjmlResult<()>`. The setter accepts only nonnegative values representable by Minecraft's `int` air-supply API and no greater than that player's current maximum.

The schema-v1 read response is 12 little-endian bytes: schema `u32` (`1`), current air `i32`, and maximum air `u32`. The `sjml_player_v1` imports are `air_supply(handle_ptr, handle_len, output_ptr, output_capacity)` and `set_air_supply(handle_ptr, handle_len, air_supply)`. Every import returns a standard SJML ABI status code.
