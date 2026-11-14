# Item furnace fuel

A module can assign a fixed burn duration to an item it registers. This adds the item to NeoForge's
built-in `data/neoforge/data_maps/item/furnace_fuels.json` data map:

```rust
use sjml::{mod_register, registry::{self, ItemDefinition}, SjmlResult};

#[mod_register]
fn register_items() -> SjmlResult<()> {
    registry::register_item("charcoal_chunk", ItemDefinition::default())?;
    registry::set_item_furnace_fuel_burn_time("charcoal_chunk", 1_600)?;
    Ok(())
}
```

Call `set_item_furnace_fuel_burn_time` during `#[mod_register]`, after staging the item, block item
form, or fluid bucket from the same module. The item path is relative to the module, like the path
passed to `register_item`. It cannot refer to another module's item or an item that is not staged.
Each item accepts one declaration. Burn time is a positive number of ticks from `1` through
`1_000_000`.

SJML emits the canonical fully qualified item ID with a `burn_time` value and merges it into the
generated server data pack's furnace-fuels map. If a package also supplies that same NeoForge data
map as raw JSON, the typed entry replaces a raw value for the exact same item ID, and a matching
direct removal is removed so it cannot delete the typed value. Other raw values and tag-based
removals are retained. The map is applied when the server loads or reloads data packs. NeoForge
servers may synchronize this data map, but the value is not available when a player connects to a
vanilla server. This API provides a static item value; it does not inspect stack NBT.

The host ABI import is
`sjml_registry_v1.set_item_furnace_fuel_burn_time(item_path_ptr: i32, item_path_len: i32, burn_time: i32) -> i32`.
It requires the module's `content.register` capability and is valid only during static registration.
