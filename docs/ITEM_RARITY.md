# Module-owned item rarity

Use `registry::set_item_rarity` during `#[mod_register]` to choose Minecraft's vanilla item
rarity for an item registered by the same package and module. The supported values are `Common`,
`Uncommon`, `Rare`, and `Epic`; an item that does not call the setter keeps vanilla's `Common`
default.

```rust
use sjml::{mod_register, registry::{self, ItemDefinition, ItemRarity}, SjmlResult};

#[mod_register]
fn register_content() -> SjmlResult<()> {
    registry::register_item("ruby", ItemDefinition::default())?;
    registry::set_item_rarity("ruby", ItemRarity::Epic)?;
    Ok(())
}
```

The setter also accepts the item form of a block registered earlier by this module. A module cannot
change another module's item or an item already present in Minecraft's registry. Each declared item
can receive one rarity declaration; setting it twice, using an undeclared item path, or calling after
the item registration phase closes is rejected. This operation requires the module's granted
`content.register` permission and runs only while the host invokes `#[mod_register]`.

The SDK type is closed to the four supported vanilla values, and the host validates the stable
integer ABI mapping (`Common = 0`, `Uncommon = 1`, `Rare = 2`, `Epic = 3`). Minecraft 1.21.1 applies
the selected value through `Item.Properties.rarity(net.minecraft.world.item.Rarity)` when the host
materializes the registered item. The existing `register_item` ABI and its defaults are unchanged.
