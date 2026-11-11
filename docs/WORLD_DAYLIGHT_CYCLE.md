# Daylight-cycle gamerule

The world API can read and change Minecraft's persisted `doDaylightCycle` gamerule
for a loaded server world:

```rust
use sjml_sdk::world::{self, WorldHandle};

fn pause_daylight_cycle(world: WorldHandle) -> sjml_sdk::SjmlResult<()> {
    let was_enabled = world::daylight_cycle_enabled(world)?;
    if was_enabled {
        world::set_daylight_cycle_enabled(world, false)?;
    }
    Ok(())
}
```

Reading requires the module to request and receive `world.read`. Changing the
rule requires both `world.read` (the world handle) and `world.write`. Both
operations are server-thread-only; client modules and static registration calls
cannot read or modify the live rule. The host delegates to Minecraft's
`GameRules` value so normal world persistence and gamerule update behavior are
preserved.

`world::set_time` changes the current absolute daylight time without changing
the rule. When `doDaylightCycle` is enabled, that time advances on world ticks;
when disabled, it remains fixed until another operation changes it. The gamerule
API does not load a world or chunk.
