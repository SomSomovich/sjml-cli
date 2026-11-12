# Absolute world game time

`world::game_time(handle)` reads the elapsed game-time clock shared by dimensions on the owning
server. Set that clock with `world::set_game_time`:

```rust
use sjml::{world, SjmlResult, WorldHandle};

fn rewind_world_clock(handle: WorldHandle) -> SjmlResult<()> {
    let current = world::game_time(handle)?;
    world::set_game_time(handle, current.saturating_sub(20))
}
```

The setter accepts absolute tick values from zero through 1,000,000,000,000. Negative values and
larger values are rejected before the host call. It changes the shared `game_time` clock only;
Minecraft's separate `day_time` value is unchanged, and can still be read or changed with
`world::day_time` and `world::set_time`.

The module must request and receive both `world.read` and `world.write`. The host validates the
module-owned world handle and performs the update on that world's logical-server thread, through
the server's canonical game-time data. The clock is shared by dimensions; the host reaches it from
the addressed, already-loaded `ServerLevel` and never loads a dimension or chunk. This is a clock
write: moving time forward can make scheduled server events due on a later tick, and moving it
backward can delay them.

The ABI import is `sjml_world_v1.set_game_time(handle_ptr, handle_len, game_time_low,
game_time_high) -> i32`. The two `i32` values carry the signed 64-bit tick value in little-endian
low/high halves; success returns zero.
