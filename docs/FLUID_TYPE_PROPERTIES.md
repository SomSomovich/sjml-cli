# Static fluid type properties

Registered simple fluids can set NeoForge's bounded static density, temperature, viscosity, and
light level while keeping the existing vanilla-style source/flowing fluid implementation:

```rust
use sjml::prelude::*;

#[sjml::mod_register]
fn register() -> SjmlResult<()> {
    let physical = FluidTypeProperties::new()
        .density(1200)?
        .temperature(900)?
        .viscosity(2400)?
        .light_level(4)?;

    registry::register_fluid_with_type_properties(
        "molten_salt",
        FluidDefinition::default(),
        physical,
    )
}
```

Use `register_fluid_with_visuals_and_type_properties` to provide package-owned still and flowing
textures as well. Existing `register_fluid` and `register_fluid_with_visuals` retain their original
signatures and use NeoForge defaults: density 1000, temperature 300, viscosity 1000, and light level
0.

The SDK accepts density from -10,000 through 10,000, temperature and viscosity from 0 through
10,000, and light level from 0 through 15. Density, temperature, and viscosity use NeoForge's
arbitrary fluid type scales. The light level is NeoForge's fluid type luminosity value used by fluid
bucket rendering; it does not set the liquid block's world light emission. The properties are
static metadata and behavior inputs for NeoForge-aware consumers. They do not add custom callbacks,
change the safe vanilla flow implementation, or replace `FluidDefinition`'s tick rate, level
decrease, slope search, and explosion resistance settings.

Registration requires the administrator-granted `content.register` capability and runs in the
module's static `mod_register` phase. The host validates each value and package-owned texture path
before staging any fluid content. A failed declaration leaves the registry staging state unchanged.
