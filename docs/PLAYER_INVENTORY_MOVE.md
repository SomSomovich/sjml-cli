# Player inventory stack moves

The Rust API `player_inventory::move_stack(player, source_slot, destination_slot, count)` moves an
exact count between two slots in the live player's inventory. It requires `entity.read` and
`inventory.write`, a valid current player session, a logical-server module, and the owning server
thread.

Slots must be different indices in `0..=40`, and the count must be `1..=64`. The source must hold
the complete requested count. The destination must be empty or contain the same registered item
with identical components. The resulting count must fit both the item's stack limit and the
destination slot; armor slots remain limited to one item.

The host copies both original stacks, changes only their counts, validates both resulting slots,
and restores both originals if a write fails or a postcondition is violated. It never creates an
item or exposes a stack through the ABI. Player inventory synchronization uses the active menu.
See also [player inventory writes](PLAYER_INVENTORY_WRITES.md).
