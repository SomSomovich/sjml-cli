# Vanilla item rarity and enchantability

During `#[mod_register]`, a module may choose a vanilla display rarity and the vanilla enchantability value for an item it declared. These settings also accept the item form of a block registered earlier by the same module.

```rust
use sjml::{mod_register, registry::{self, ItemDefinition, ItemRarity}, SjmlResult};

#[mod_register]
fn register_content() -> SjmlResult<()> {
    registry::register_item("ruby", ItemDefinition::default())?;
    registry::set_item_rarity("ruby", ItemRarity::Epic)?;
    registry::set_item_enchantment_value("ruby", 22)?;
    Ok(())
}
```

`set_item_rarity` accepts `Common`, `Uncommon`, `Rare`, and `Epic`. Items default to vanilla `Common` rarity. Each item accepts at most one rarity declaration.

`set_item_enchantment_value` accepts `0..=255`. The value is passed to Minecraft's `Item.Properties.enchantmentValue`; zero retains the vanilla non-enchantable behavior. This configures vanilla enchantability and does not declare custom enchantments.

Both calls require the module's granted `content.register` permission and run only while the host invokes `#[mod_register]`, before the item registry closes. They reject undeclared items, items owned by another module, duplicate declarations, and invalid values. The host applies the selected properties when materializing the item.
