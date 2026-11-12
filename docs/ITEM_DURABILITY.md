# Item durability

An item module can declare a vanilla max-damage value for one of its own
registered standalone items during `#[mod_register]`:

```rust
use sjml::{mod_register, registry::{self, ItemDefinition}, SjmlResult};

#[mod_register]
fn register_items() -> SjmlResult<()> {
    registry::register_item(
        "bronze_pick",
        ItemDefinition { max_stack_size: 1, fire_resistant: false },
    )?;
    registry::set_item_max_damage("bronze_pick", 384)?;
    Ok(())
}
```

`set_item_max_damage` accepts `1..=1_000_000`. The item must be declared by
the same module with `max_stack_size = 1`; block items, fluid buckets,
stackable items, missing items, and duplicate declarations are rejected. The
host installs Minecraft's standard `MAX_DAMAGE` item property, so ordinary
`ItemStack` damage, repair, and break behavior applies without a custom Java
item class. The current API configures capacity; it does not add a custom
repair ingredient or use behavior.

The ABI import is
`sjml_registry_v1::set_item_max_damage(path_pointer: i32, path_length: i32, max_damage: i32) -> i32`.
The host copies and validates the UTF-8 path, enforces the existing
`content.register` grant and static registration phase, and keeps the
declaration scoped to its package and module.
