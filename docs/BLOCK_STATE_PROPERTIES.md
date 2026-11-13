# Registered block-state properties

Modules can declare bounded Boolean and integer state properties for their
own registered blocks during `#[mod_register]`. The host stages the properties
and installs them before Minecraft freezes the block registry. Every state is
then an ordinary Minecraft `BlockState`, so existing world property APIs can
inspect and update it.

```rust
use sjml::{mod_register, registry::{self, BlockDefinition, BlockStateProperty}};

#[mod_register]
fn register_content() -> sjml::SjmlResult<()> {
    registry::register_block("machine", BlockDefinition::default())?;
    registry::add_block_state_property(
        "machine",
        &BlockStateProperty::boolean("active", false)?,
    )?;
    registry::add_block_state_property(
        "machine",
        &BlockStateProperty::integer("charge", 0, 7, 2)?,
    )?;
    Ok(())
}
```

Declare the block first, then add properties to that block in the same
registration callback. Property registration requires the module's
`content.register` grant. Names contain 1–32 lowercase ASCII letters, digits,
or underscores. A block may declare up to seven properties and have at most
128 possible states. Integer properties contain 2–16 inclusive values in
`-128..=127`; their default must be in the declared range. Boolean defaults
are supplied as Rust `bool` values. The host also limits the total staged
registered-block state space to 16,384 variants.

After registration, use `world::block_properties` to read the string-valued
state properties of a block in an already-loaded chunk and
`world::set_block_properties` to update selected properties. These world calls
require `world.read` and, for writes, `world.write`; they do not load chunks.
The write API validates the target block ID and property values before
changing the world. See [block-state snapshots](WORLD_BLOCK_PROPERTIES.md)
and [guarded block-state writes](WORLD_BLOCK_PROPERTIES_WRITE.md).

The declaration is static: property definitions and defaults cannot be added
or changed after registry freeze. Supported property kinds are Boolean and
bounded integer only. They do not define state-dependent geometry, custom
placement rules, callbacks, synchronization code, or arbitrary Java property
types. See [block shapes](WORLD_BLOCK_SHAPES.md) and
[block behavior](WORLD_BLOCK_BEHAVIOR.md) for the separate fixed registration
options.
