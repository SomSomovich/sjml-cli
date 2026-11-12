# Player inventory slot writes

Server-side modules can replace one live player's inventory slot with a registered item or clear
it through `sjml::player_inventory::set_slot`:

```rust
use sjml::PlayerInventorySlotUpdate;

sjml::player_inventory::set_slot(
    player,
    0,
    PlayerInventorySlotUpdate::Set {
        item_id: "minecraft:diamond",
        count: 4,
    },
)?;
sjml::player_inventory::set_slot(player, 40, PlayerInventorySlotUpdate::Clear)?;
```

To change the durability of an existing damageable item while preserving its other components,
use `set_damage`:

```rust
sjml::player_inventory::set_damage(player, 0, 7)?;
```

The slot must contain a nonempty damageable stack. The damage value must be less than the item's
maximum durability and no greater than 65,535; the host rejects a value that would break the item.
The update is applied to a copy and synchronized through the active player menu.

Slot indices match the detached snapshot: `0..=8` hotbar, `9..=35` main storage, `36..=39`
armor (feet through head), and `40` offhand. Armor slots accept one item. The host rejects
indices outside `0..=40`, unknown or noncanonical registry IDs, `minecraft:air`, zero counts for
`Set`, counts beyond the item's registered stack size or the slot limit, and item IDs larger than
256 UTF-8 bytes. `Clear` is the only empty-slot encoding.

For `set_slot`, the guest supplies only an item registry ID and count. The host creates a fresh
default item stack; this operation does not accept item components, custom names, NBT, or component
JSON. It validates the entire update before replacing the slot, then marks the player inventory
changed and broadcasts changes through the player's active menu so open containers and the player
inventory stay in sync. `set_damage` changes only the durability component of an existing stack;
the host preserves its other components.

Modules must declare and receive both `entity.read` and `inventory.write`. The operation is only
available on the logical server, uses the owning server thread, and re-resolves the live login
session for each call. Foreign or stale handles, client contexts, static registration, and calls
off-thread are denied. The ABI imports are
`sjml_inventory_v1.set_player_slot(player_ptr, player_len, slot, item_id_ptr, item_id_len, count) -> i32`
and `sjml_inventory_v1.set_player_damage(player_ptr, player_len, slot, damage) -> i32`.
