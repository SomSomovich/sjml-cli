# Giving ordinary items to a player

Server modules can give registered items to a live player's inventory with
`sjml::player_inventory::give_item`:

```rust
let inserted = sjml::player_inventory::give_item(player, "minecraft:diamond", 12)?;
let remaining = 12 - inserted;
```

The item ID must be canonical and registered. The host creates the item's
ordinary default stack and accepts requests from 1 through 4,096 items. It
fills matching stacks first, then empty hotbar and main-storage slots
(`0..=35`). It does not use armor or offhand slots, and it merges only when
the item and all existing stack components match. No item components or NBT
can be supplied by the guest.

The return value is the count inserted. It may be lower than requested when
space runs out; uninserted items remain with the caller, and the host does not
drop or discard them. A full inventory returns `0` without changing any slot.
The complete slot plan is checked before it is committed, and successful
changes are synchronized through the player's active menu.

The call requires declared and administrator-granted `entity.read` and
`inventory.write`, a live player handle, and a logical-server main thread.
Client-only modules and static-registration callbacks cannot use it. See
[player inventory writes](PLAYER_INVENTORY_WRITES.md) for direct slot updates.

The import is
`sjml_inventory_v1.give_player_item(player_ptr, player_len, item_id_ptr, item_id_len, count, inserted_ptr, inserted_capacity) -> i32`.
The output span must be exactly four writable bytes; on success it receives
the inserted count as a little-endian `i32`.
