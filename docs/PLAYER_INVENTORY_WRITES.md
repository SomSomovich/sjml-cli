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

Slot indices match the detached snapshot: `0..=8` hotbar, `9..=35` main storage, `36..=39`
armor (feet through head), and `40` offhand. Armor slots accept one item. The host rejects
indices outside `0..=40`, unknown or noncanonical registry IDs, `minecraft:air`, zero counts for
`Set`, counts beyond the item's registered stack size or the slot limit, and item IDs larger than
256 UTF-8 bytes. `Clear` is the only empty-slot encoding.

The guest supplies only an item registry ID and count. The host creates a fresh default item stack;
the API does not accept item components, custom names, NBT, or component JSON. It validates the
entire update before replacing the slot, then marks the player inventory changed and broadcasts
changes through the player's active menu so open containers and the player inventory stay in sync.

Modules must declare and receive both `entity.read` and `inventory.write`. The operation is only
available on the logical server, uses the owning server thread, and re-resolves the live login
session for each call. Foreign or stale handles, client contexts, static registration, and calls
off-thread are denied. The ABI import is
`sjml_inventory_v1.set_player_slot(player_ptr, player_len, slot, item_id_ptr, item_id_len, count) -> i32`.
