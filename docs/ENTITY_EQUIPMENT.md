# Loaded entity equipment snapshots

`entity::equipment(handle)` returns a detached snapshot of the six vanilla equipment slots for one loaded, living, non-player entity:

1. Main hand
2. Offhand
3. Head
4. Chest
5. Legs
6. Feet

Each slot is `Option<EntityEquipmentStack>`. An occupied slot exposes only the canonical item registry ID, stack count, current durability damage, and maximum durability damage. An empty slot is `None`. Items without durability report `damage = 0` and `max_damage = 0`.

The module must request and receive `entity.read`. Each call checks that the entity handle belongs to the current module and execution context, still identifies the same live entity instance, and resolves to a loaded `LivingEntity`; it runs on the owning logical-server thread. The read does not load chunks.

The snapshot does not expose item handles, inventory contents, NBT, data components, capabilities, or mutable item access. This API is for decisions based on what a living entity currently holds or wears; it cannot copy, modify, or serialize an equipped stack.

The schema-v1 response is bounded to 1,666 bytes. It starts with little-endian `u32(1)`, followed by six option tags in the order above. A present slot contains little-endian `u32(1)` for its stack schema, a length-prefixed UTF-8 canonical item ID, then little-endian `u32 count`, `u32 damage`, and `u32 max_damage`. Item IDs are at most 256 bytes, counts are `1..=4096`, damage values are at most 1,000,000, and `damage <= max_damage`.

```rust
use sjml::{entity, SjmlResult};

fn equipped_item_ids(entity_handle: sjml::EntityHandle) -> SjmlResult<Vec<String>> {
    let equipment = entity::equipment(entity_handle)?;
    Ok([
        equipment.main_hand,
        equipment.off_hand,
        equipment.head,
        equipment.chest,
        equipment.legs,
        equipment.feet,
    ]
    .into_iter()
    .flatten()
    .map(|stack| stack.item_id)
    .collect())
}
```

The `sjml_entity_v1.equipment` import accepts the 24-byte entity handle and a writable output span. Static registration, client execution, stale/foreign handles, non-living entities, and missing grants fail closed. The SDK reserves the full bounded output size before calling the host, so a short output buffer cannot return a partial snapshot.

`EntityEquipmentCodecTest`, `EntityEquipmentWriteCodecTest`, `EntityReadImportsTest`, `EntityHandleRegistryTest`, the SDK contract tests, and the guest-driven `EntityEquipmentGameTests` cover schema order, empty slots, values, permissions, live handles, and server execution.

## Adding one default item to an empty slot

`entity::try_equip_default(handle, slot, item_id)` creates one default stack from an item registered
by that exact package and module, then places it into one fixed slot. The module must request and
receive both `entity.read` and `entity.equipment.write`. The target must still be a loaded living
non-player entity on the owning server thread. IDs must be canonical `namespace:path` values owned
by the module; vanilla items and items registered by other modules are rejected. The guest cannot
provide a count, NBT, or custom data components.

The call returns `true` when it equips the stack and `false` when that slot already contains an
item. An occupied stack is never replaced or discarded. `entity::clear_equipment(handle, slot)`
clears a slot through Minecraft's `LivingEntity.setItemSlot` path and returns `true` when a stack was
removed or `false` when it was already empty. Both calls use the same permission and six-slot enum.
The clear call triggers vanilla equipment synchronization; no arbitrary stack or item data crosses
the ABI.

```rust
use sjml::{entity, EntityEquipmentSlot, SjmlResult};

fn equip_zombie(mob: sjml::EntityHandle) -> SjmlResult<bool> {
    entity::try_equip_default(mob, EntityEquipmentSlot::MainHand, "my_package:my_module/iron_sword")
}
```

`sjml_entity_v1.set_equipment` requires `entity.read` and `entity.equipment.write` and accepts a schema-v1 request containing the 24-byte entity handle,
slot code (`1` main hand, `2` offhand, `3` head, `4` chest, `5` legs, `6` feet), and a canonical item
ID capped at 256 UTF-8 bytes and owned by the calling package/module. `clear_equipment` accepts the
same schema version, handle, and slot code with no item ID. Both return exactly five bytes: schema
`u32(1)` followed by a `0` or `1` boolean tag. The host checks the output span before reaching the
entity provider. Static registration links the bounded imports but denies live mutation.

For the separate Mob-only per-slot drop-chance API and its capability, see
[`ENTITY_EQUIPMENT_DROP_CHANCES.md`](ENTITY_EQUIPMENT_DROP_CHANCES.md).
