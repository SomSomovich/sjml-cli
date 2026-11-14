# Per-state flammability and fire spread speed

Use `BlockStateBehavior::flammability` and `BlockStateBehavior::fire_spread_speed` to configure
Minecraft's values for one exact declared block state:

```rust
registry::add_block_state_property("timber", &BlockStateProperty::boolean("treated", false)?)?;
registry::set_block_state_behavior(
    "timber",
    &[("treated", "true")],
    registry::BlockStateBehavior::new()
        .flammability(72)?
        .fire_spread_speed(45)?,
)?;
```

Both values are bounded integers from `0` through `300`. A value of `0` disables the matching
flammability or spread contribution for that state. Declare all state properties first and provide
each property once in the selector. Calls may add separate fields to the same state; assigning the
same field twice is rejected. States and fields left unset keep the block's existing behavior.

The host calls NeoForge's state-aware `IBlockExtension.getFlammability` and
`IBlockExtension.getFireSpreadSpeed` hooks. Minecraft uses flammability when fire checks whether a
block is consumed and spread speed when calculating nearby fire growth. Values are staged during
`#[mod_register]` for a same-module registered block with `content.register`; immutable lookup tables
serve runtime queries, so guest code never runs while fire updates.

The payload advances from schema v4 to v5 by appending two little-endian `i32` values after the
piston reaction field. The host continues to decode schema-v1 through v4 payloads.
