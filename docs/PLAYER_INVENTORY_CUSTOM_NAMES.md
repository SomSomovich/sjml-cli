# Player inventory item custom names

`sjml::player_inventory::set_item_custom_name` and `clear_item_custom_name` change the vanilla
`CUSTOM_NAME` data component on an existing player inventory stack. They do not replace the stack:
the item identity, count, durability, and every unrelated component are preserved. Empty slots are
rejected, and the API cannot create or clone an item.

The module must run on the logical server and declare and receive both `entity.read` and
`inventory.write`. `inventory.read` is not required just to set or clear a name. Host calls resolve
the live player session and run on its owning server thread. The updated inventory is marked changed
and synchronized through the player's active menu.

```rust
use sjml::player_inventory;

player_inventory::set_item_custom_name(player, 0, "Café 🪨")?;
player_inventory::clear_item_custom_name(player, 0)?;
```

Names are plain literal text. They must contain 1–256 Unicode scalar values, use at most 1,024
UTF-8 bytes, and contain no control characters. An empty string is invalid; call
`clear_item_custom_name` to remove the component.

Player inventory slots use the Minecraft indices: `0..=8` hotbar, `9..=35` main storage, `36..=39`
armor (feet through head), and `40` offhand. Setting or clearing a name requires a nonempty stack at
the selected slot. Clearing an already absent name is allowed and leaves the other stack data intact.

Example manifest permissions:

```toml
[[modules]]
id = "main"
side = "server"
permissions = ["entity.read", "inventory.write"]
```
