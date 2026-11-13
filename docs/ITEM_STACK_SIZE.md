# Static item stack size

Use `registry::set_item_max_stack_size` from a module's `#[mod_register]` callback to set a registered standalone item's vanilla stack limit:

```rust
use sjml_sdk::{mod_register, registry};

#[mod_register]
fn register() -> sjml_sdk::SjmlResult<()> {
    registry::register_item("portable_crate", registry::ItemDefinition::default())?;
    registry::set_item_max_stack_size("portable_crate", 16)?;
    Ok(())
}
```

The accepted range is `1..=99`. Minecraft 1.21.1's `net.minecraft.world.item.Item.Properties.stacksTo(int)` stores this static item property. NeoForge 21.1.265's `RegisterEvent` for `Registries.ITEM` owns final registration; SJML materializes the `Item` with the configured `Properties` before passing it to that event. Vanilla `ItemStack` inventory behavior then reads the configured maximum.

The guest module declares the value and owns the target item; the host validates and stages the override during `#[mod_register]`. The path is relative to the calling module, and the item must have been declared by that same module with `registry::register_item`. The host rejects a missing item, a block item, a declaration made by a different module, a second override for the same item, and calls outside static registration. If module registration fails before registry events, package rollback removes both the item and its stack-size override.

An item with vanilla durability must have stack size `1`. SJML enforces this in either declaration order: setting a larger stack size after `registry::set_item_max_damage` fails, and setting durability after a stack-size override above `1` fails. A stack-size override of `1` is compatible with durability. Do not use this API to change a stack limit after the item registry has been emitted.

The equivalent host ABI import is `sjml_registry_v1.set_item_max_stack_size(item_path_ptr: i32, item_path_len: i32, max_stack_size: i32) -> i32`; it requires the existing `content.register` capability.
