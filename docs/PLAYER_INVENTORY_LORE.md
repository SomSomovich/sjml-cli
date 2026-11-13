# Player inventory lore

The SDK can read, replace, or clear the plain-text view of Minecraft's vanilla lore component on
one existing player inventory stack:

```rust
let lines = sjml::player_inventory::read_lore(player, 0)?;
sjml::player_inventory::set_lore(
    player,
    0,
    &["Café 🪨".to_owned(), "§cshown as literal text".to_owned()],
)?;
sjml::player_inventory::clear_lore(player, 0)?;
```

Reads flatten the existing `DataComponents.LORE` lines to detached strings. The API does not expose
component JSON, styles, click events, hover events, or Minecraft component objects. Writes create
only literal `Component` values, so section-sign text is passed as text and is not parsed into
formatting.

Lore is limited to eight lines. Each line can contain at most 256 UTF-8 bytes and 256 Unicode scalar
values, with no control characters; a blank line is valid. Existing lore outside these bounds fails
with `PayloadTooLarge` rather than being truncated. An empty slice is rejected; use
`clear_lore` to remove the component. Reading an empty slot or a stack without lore returns an
empty vector. Set and clear require an existing nonempty stack.

The host updates a copy of the current stack and changes only `DataComponents.LORE`. Item identity,
count, damage, custom name, enchantments, and unrelated components remain intact. It marks the
inventory changed and synchronizes the player's active menu.

All operations use the vanilla player slot indices: `0..=8` hotbar, `9..=35` main storage,
`36..=39` armor, and `40` offhand. Reads require `entity.read` and `inventory.read`. Writes require
`entity.read` and `inventory.write`. These APIs are available only to logical-server `server` or
`both` modules, run on the owning server thread, and resolve a live player session on each call.

The host imports are `sjml_inventory_v1.read_player_lore`, `set_player_lore`, and
`clear_player_lore`. Their schema-v1 payload and function signatures are specified in the
[ABI reference](ABI.md#plain-text-lore-on-player-inventory-stacks).
