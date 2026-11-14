# Loaded biome climate reads

Server modules can inspect climate settings for the biome at an already-loaded
world position:

```rust
let climate = world::biome_climate_at(world, BlockPos { x, y, z })?;
if let Some(climate) = climate {
    // climate.biome_id, base_temperature, downfall, has_precipitation,
    // temperature_modifier, and precipitation_at_position
}
```

The call requires `world.read` and a live server `WorldHandle`. It returns
`None` when the position is outside build height or its chunk is not already
loaded. The call never loads a chunk. The returned values are detached from
Minecraft objects.

`base_temperature` and `downfall` are the biome's modified registry climate
settings. `temperature_modifier` reports Minecraft's `None` or `Frozen` rule.
`precipitation_at_position` reports `None`, `Rain`, or `Snow` for the queried
position; the Y coordinate can therefore affect the result. It describes the
biome's precipitation kind, not whether it is currently raining in the world.
