# Changing an existing player stack count

`player_inventory::set_count` changes only the count of a nonempty stack already
in a live player's inventory:

```rust
player_inventory::set_count(player, 0, 3)?;
```

Slot indices are the same as `player_inventory::read`: hotbar `0..=8`, main
storage `9..=35`, armor `36..=39`, and offhand `40`. The requested count must
be positive and no greater than the smaller of the existing item's maximum
stack size and the slot limit. Armor slots remain limited to one item.

The host copies the existing stack and changes only its count. The item and all
data components remain unchanged. Empty slots, invalid indices, and values
above either limit fail without creating a stack, splitting it into other
slots, or changing inventory state. Successful changes mark the inventory
dirty and synchronize the active player menu.

This server-only operation requires declared and administrator-granted
`entity.read` and `inventory.write`, a live player handle, and the owning server
thread. Static module registration links the import when permitted but denies
its invocation.
