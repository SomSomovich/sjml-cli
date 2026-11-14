# Per-stack glint override in player inventories

Use `sjml::player_inventory` to force or suppress Minecraft's foil effect on a
single existing player-inventory stack, or clear the override to restore the
item's normal behavior:

```rust
sjml::player_inventory::set_item_glint_override(player, 0, true)?;  // force foil
sjml::player_inventory::set_item_glint_override(player, 0, false)?; // suppress foil
sjml::player_inventory::clear_item_glint_override(player, 0)?;      // use vanilla behavior
```

`true` forces the foil effect even without enchantments. `false` suppresses it
even when enchantments would normally show foil. Clearing removes the explicit
override, so Minecraft decides from the remaining components and item behavior.
The operation changes only `DataComponents.ENCHANTMENT_GLINT_OVERRIDE`; it does
not add or remove enchantments.

Each call accepts a slot from `0..=40` in the standard player inventory layout
and requires a nonempty stack. It copies the existing stack, preserves the item,
count, durability, and every component other than the glint override, marks the
inventory changed, and synchronizes the active menu. It never replaces the slot
with an arbitrary item stack.

Modules must declare and receive both `entity.read` and `inventory.write`. The
calls are available to server and `both` modules, run on the owning server
thread, and resolve the current live player session each time. Empty slots,
stale handles, client contexts, static registration, and off-thread calls fail
closed.

The host imports are
`sjml_inventory_v1.set_player_item_glint_override(player_ptr, player_len, slot, enabled) -> i32`
and
`sjml_inventory_v1.clear_player_item_glint_override(player_ptr, player_len, slot) -> i32`.
`enabled` is exactly `0` or `1`; success returns status `0`, and failures use
the standard ABI error codes.

This per-stack API is distinct from
[`registry::set_item_glint_override`](ITEM_GLINT_OVERRIDE.md), which sets the
default for newly created stacks of an SJML-registered item.
