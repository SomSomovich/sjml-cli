# World time API

The Rust SDK exposes two separate clocks for an already loaded server world:

```rust
let game_ticks = world::game_time(handle)?;
let day_ticks = world::day_time(handle)?;
world::set_time(handle, 12_000)?;
```

`game_time` is the world's elapsed server tick count. It advances with world
ticks regardless of the daylight-cycle gamerule. `day_time` is Minecraft's
absolute daylight-cycle time; it may stop advancing while `doDaylightCycle` is
disabled. `set_time` changes only `day_time`, not `game_time` or the gamerule.
With daylight cycling enabled, the new value advances again on the next world
tick.

Reading requires `world.read`. Changing time requires both `world.read` to obtain
the scoped world handle and `world.write` to mutate it. The API runs on the
owning logical server thread and accepts absolute values from zero through
1,000,000,000,000 ticks (more than 1,500 years). Negative values and values
above that bound are rejected before Minecraft is changed.

The host resolves the module-owned handle against its already loaded dimension;
neither lookup nor mutation loads a dimension or chunk. The existing
`sjml_world_v1.game_time` import remains available for compatibility. New
imports are `sjml_world_v1.day_time` and `sjml_world_v1.set_time`.
