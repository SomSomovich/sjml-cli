# Item crafting remainder

A module can assign one static vanilla crafting remainder to a standalone item
that it registered itself. The example below makes a custom ingredient return a
bucket, like a milk bucket does in vanilla recipes:

```rust
use sjml::{mod_register, registry::{self, ItemDefinition}, SjmlResult};

#[mod_register]
fn register_items() -> SjmlResult<()> {
    registry::register_item("portable_container", ItemDefinition::default())?;
    registry::set_item_crafting_remainder("portable_container", "minecraft:bucket")?;
    Ok(())
}
```

Call `set_item_crafting_remainder` during `#[mod_register]`, after
`register_item`, with the `content.register` capability granted. The owner must
be a standalone item declared by the same module; block items and items owned
by another module cannot receive this property. Only one remainder can be
declared per item.

The remainder argument must be a canonical, fully qualified item registry ID
of at most 256 UTF-8 bytes, such as `minecraft:bucket`, that already resolves
in the item registry when the registration callback runs. This ordering rule
avoids referring to an item that has not been created yet. Vanilla items and
any other item already present at that point are supported. An item registered
later in a registry event, including another SJML-staged item, cannot be used.
Unknown, non-canonical, or
unqualified IDs are rejected.

The host resolves the ID to the registered `Item` and applies it through
`Item.Properties.craftRemainder` before materializing the owner item. The API
does not create or mutate `ItemStack`s and does not expose components or NBT.
Minecraft's normal crafting remainder behavior returns the configured item
when a matching recipe consumes the ingredient. The SDK GameTest example
registers an item with a bucket remainder; `ItemCraftingRemainderGameTests`
checks it alongside vanilla milk-bucket behavior.

The ABI import is
`sjml_registry_v1.set_item_crafting_remainder(item_path_ptr, item_path_len, remainder_item_id_ptr, remainder_item_id_len) -> i32`.
