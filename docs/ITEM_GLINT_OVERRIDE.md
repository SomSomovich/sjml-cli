# Item glint override

An SJML module can set the vanilla enchantment-glint default for one of its registered items during `#[mod_register]`:

```rust
use sjml::prelude::*;

#[mod_register]
fn register_content() -> SjmlResult<()> {
    registry::register_item("glowing_token", ItemDefinition::default())?;
    registry::set_item_glint_override("glowing_token", true)?;
    Ok(())
}
```

`true` forces the vanilla foil effect, even when the stack has no enchantments. `false` suppresses the effect, including on enchanted stacks. If no override is declared, Minecraft keeps its normal behavior. The value is stored as the item's default `ENCHANTMENT_GLINT_OVERRIDE` data component; it does not edit stacks that already exist.

The setter accepts an item or item form declared earlier by the same module. This includes standalone items, registered block items, spawn eggs, and fluid buckets. Each item accepts one declaration. The item must be registered before the module's item-registration phase closes, and the module needs the existing `content.register` capability.

The API only selects the display flag. It does not add enchantments or change item rarity, enchantability, or any per-stack data.
