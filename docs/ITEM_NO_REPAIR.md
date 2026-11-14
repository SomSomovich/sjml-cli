# Disable vanilla repairability

Use `registry::set_item_no_repair` to set Minecraft's vanilla repairability
flag to false for one of your durable standalone items:

```rust
use sjml::prelude::*;

#[mod_register]
fn register_content() -> SjmlResult<()> {
    registry::register_item(
        "sealed_tool",
        ItemDefinition {
            max_stack_size: 1,
            fire_resistant: false,
        },
    )?;
    registry::set_item_max_damage("sealed_tool", 512)?;
    registry::set_item_no_repair("sealed_tool")?;
    Ok(())
}
```

Call `set_item_no_repair` after `set_item_max_damage` for the same standalone
item. The item must be damageable, owned by the current module, and may receive
one declaration. This requires the existing `content.register` permission and
must run during `#[mod_register]` before the item registry phase closes.

SJML applies Minecraft's `Item.Properties.setNoRepair()` before constructing the
item. The item still takes durability damage and can break; Minecraft's
`Item.isRepairable(stack)` returns false. Items without this declaration keep
their normal repair behavior.
