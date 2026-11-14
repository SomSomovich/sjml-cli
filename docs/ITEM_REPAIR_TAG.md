# Configure an item's repair ingredient tag

Use `registry::set_item_repair_tag` to make Minecraft accept members of an item
tag as the repair ingredient for one of your damageable standalone items:

```rust
use sjml::prelude::*;

#[mod_register]
fn register_content() -> SjmlResult<()> {
    registry::register_item(
        "wooden_tool",
        ItemDefinition {
            max_stack_size: 1,
            fire_resistant: false,
        },
    )?;
    registry::set_item_max_damage("wooden_tool", 256)?;
    registry::set_item_repair_tag("wooden_tool", "minecraft:planks")?;
    Ok(())
}
```

Call it during `#[mod_register]`, after registering the item and setting its
positive max damage. The item must be a standalone item registered by the same
module. The existing `content.register` permission applies, the item registry
phase must still be open, and each item accepts one repair tag. The tag ID must
be canonical `namespace:path` text of at most 256 UTF-8 bytes. It cannot be
combined with `registry::set_item_no_repair`.

Minecraft 1.21.1 has no `Item.Properties.repairable(TagKey<Item>)` setting. SJML
stores the tag during registration and checks candidates in the item's vanilla
`Item.isValidRepairItem` hook, which `AnvilMenu` uses for damageable first
inputs. For example, `minecraft:planks` accepts oak planks and the other current
members of that tag. The tag must be supplied by Minecraft, a mod, or a data
pack. SJML does not create it or change existing `ItemStack`s. Without this
setting, vanilla repair behavior is unchanged.
