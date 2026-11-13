# Per-state emitted light

A module can assign emitted light to exact states of its registered blocks. Add
all state properties first, then call
`registry::set_block_state_light_level` during `#[mod_register]`:

```rust
use sjml::prelude::*;

#[mod_register]
fn register_content() -> SjmlResult<()> {
    registry::register_block("signal_lamp", BlockDefinition::default())?;
    registry::add_block_state_property(
        "signal_lamp",
        &BlockStateProperty::boolean("powered", false)?,
    )?;
    registry::set_block_state_light_level("signal_lamp", &[("powered", "false")], 2)?;
    registry::set_block_state_light_level("signal_lamp", &[("powered", "true")], 14)
}
```

The state selector must contain every property declared on the block exactly
once. Names and values must use the canonical names exposed by the property;
Boolean values are `true` or `false`, and integer values are their decimal
strings. The host rejects missing or unknown properties, unsupported values,
duplicate property names, repeated exact-state declarations, and light levels
outside `0..=15`. A block can declare at most seven properties and 128 possible
state combinations; the host also bounds the total number of overrides.

States without an override keep the block's base level from
`BlockBehavior::light_level`. Overrides are static declarations. NeoForge
builds an immutable exact-state lookup into the block's state definitions, so
normal state changes, rendering, lighting, and neighbor updates do not call
guest code. `world::set_block_properties` can switch a loaded block between
these already-defined states. The registered block and its state properties
must belong to the module making the declarations.

See [block-state properties](BLOCK_STATE_PROPERTIES.md), the
[ABI registry imports](ABI.md#static-registry-declarations), and
[`BlockStateLightLevelGameTests`](../src/main/java/dev/somuch/sjml/neoforge/gametest/BlockStateLightLevelGameTests.java)
for the static declaration, wire format, and live server check.
