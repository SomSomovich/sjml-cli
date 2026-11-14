# Registered item fire resistance

Use `registry::set_item_fire_resistant` during `#[mod_register]` to choose
whether a module-owned standalone item or registered block item is fire
resistant:

```rust
use sjml::prelude::*;

#[mod_register]
fn register_content() -> SjmlResult<()> {
    registry::register_item("fireproof_token", ItemDefinition::default())?;
    registry::set_item_fire_resistant("fireproof_token", true)?;

    registry::register_block("fireproof_ore", BlockDefinition::default())?;
    registry::set_item_fire_resistant("fireproof_ore", true)?;
    Ok(())
}
```

The setter adds Minecraft's vanilla `FIRE_RESISTANT` default data component to
the registered item. Dropped item entities then report fire immunity and ignore
fire damage. Passing `false` removes the default; for standalone items this
overrides `ItemDefinition::fire_resistant`, including a `true` value there. An
item with no setter call keeps its declared default.

Call the setter once, after declaring the item or block in the same module and
before the item registry phase closes. It requires the existing
`content.register` permission. The setting is applied while the host builds the
item's `Item.Properties`; it does not modify arbitrary existing stacks.
