# Registered block behavior

Modules with `content.register` can assign bounded vanilla movement and physical
properties during `#[mod_register]`:

```rust
use sjml::prelude::*;

#[mod_register]
fn register_content() -> SjmlResult<()> {
    registry::register_block_with_behavior(
        "ruby_lamp",
        BlockDefinition::default(),
        BlockBehavior::new()
            .friction(0.91)?
            .speed_factor(0.4)?
            .jump_factor(1.5)?
            .light_level(13)?
            .ignited_by_lava()
            .push_reaction(BlockPushReaction::PushOnly)
            .sound(BlockSound::Amethyst),
    )
}
```

`BlockBehavior::new()` starts with vanilla-like friction `0.6`, speed and jump
factors `1.0`, zero emitted light, normal piston behavior, and stone sounds.
Friction is finite and limited to `0.0..=2.0`; speed and jump factors are
finite and limited to `0.0..=4.0`; light is `0..=15`. The piston reactions are
`Normal`, `Destroy`, `Block`, `Ignore`, and `PushOnly`. Sound is selected from
the bounded `BlockSound` list: stone, wood, gravel, grass, metal, glass, sand,
wool, snow, slime, honey, netherite, or amethyst. `ignited_by_lava` enables the
vanilla lava-ignition property.

`register_block` retains its existing import and defaults. Call
`registry::set_block_behavior` after it during the same registration callback
to change the behavior. The settings apply uniformly to every block state;
state-dependent callbacks and arbitrary sound types are not exposed. See the
[SDK guide](SDK_GUIDE.md#static-content-and-creative-tab-declarations) and
[ABI reference](ABI.md#static-registry-declarations).
