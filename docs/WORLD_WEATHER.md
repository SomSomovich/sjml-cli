# World weather API

The `sjml::world` module exposes detached weather snapshots and bounded weather presets for an
already-loaded logical-server dimension. The SDK never returns a Minecraft object.

Request `world.read` to inspect the current state. Request both `world.read` and `world.write` to
change it, and ask the server administrator to grant those capabilities to the module.

```rust
use sjml::prelude::*;

fn set_overworld_weather() -> SjmlResult<()> {
    let Some(overworld) = world::get("minecraft:overworld")? else {
        return Ok(()); // The dimension is not loaded in this server context.
    };

    let current = world::weather(overworld)?;
    if !current.raining {
        world::set_weather(overworld, world::WeatherCondition::Rain, 24_000)?;
    }
    Ok(())
}
```

`world::weather(handle)` returns `WeatherSnapshot` with `raining`, `thundering`,
`clear_time`, `rain_time`, and `thunder_time`. Timers are tick counts read from the dimension's
vanilla level data; inactive timers are normally zero. The schema is versioned and little-endian:
`u32 schema = 1`, two strict boolean bytes, then three `u32` timers in that field order.

`world::set_weather(handle, condition, duration_ticks)` accepts `WeatherCondition::Clear`,
`Rain`, or `Thunder`. The duration must be from 1 through 1,728,000 ticks (24 in-game hours).
Clear weather sets the clear timer; rain and thunder set the vanilla weather transition timer.
The host uses `ServerLevel.setWeatherParameters`, the same Minecraft API used by the vanilla
weather command.

Both calls validate the module-owned world handle and run on its logical-server thread. The
dimension must already be loaded. Weather access does not request chunks or load dimensions.
The setter is unavailable during static registration, in client runtimes, or without the full
capability intersection. Invalid condition tags and out-of-range durations return
`InvalidArgument`.

## ABI shape

Both imports use `sjml_world_v1`. `weather` takes four `i32` arguments: handle pointer, handle
length, output pointer, and output capacity. It writes the schema-v1 snapshot above. `set_weather`
takes four `i32` arguments: handle pointer, handle length, condition tag (`0=clear`, `1=rain`,
`2=thunder`), and duration in ticks. Both return zero on success or a stable nonzero ABI error.
