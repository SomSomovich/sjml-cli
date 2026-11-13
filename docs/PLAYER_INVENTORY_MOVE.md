# Moving player inventory stacks

Use `player_inventory::move_stack` to move an exact count between two slots of one live player:

```rust
player_inventory::move_stack(player, 0, 9, 4)?;
```

The module must request and receive `entity.read` and `inventory.write`. The call is server-only,
runs on the owning server thread, and resolves the current live player session for each request.
Source and destination slots must be different indices in `0..=40`; the count is bounded to
`1..=64`. The source must contain that many items. The destination must be empty or contain the
same item with identical components, and the final stack must fit its item and slot limits. The
armor slots remain limited to one item.

The operation changes only counts and preserves the stack's other components. Both slots are
preflighted before writes, then checked against the expected result. If a commit fails or violates
the postcondition, the host restores and verifies both original stacks. No item value, component,
or NBT crosses the ABI.

The host import is `sjml_inventory_v1.move_player_stack(player_ptr, player_len, source_slot,
destination_slot, count) -> i32`. See [player inventory slot writes](PLAYER_INVENTORY_WRITES.md) for
the other bounded write operations.
