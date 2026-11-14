# Mob persistence and item pickup

Loaded living non-player `Mob` instances expose two bounded lifecycle flags:

```rust
let persistent = entity::is_persistence_required(mob_handle)?;
entity::set_persistence_required(mob_handle)?;

let can_pick_up = entity::can_pick_up_loot(mob_handle)?;
entity::set_can_pick_up_loot(mob_handle, true)?;
```

All calls require `entity.read`, a live Mob handle, and the owning logical-server thread. They do
not allocate handles, load chunks, expose Minecraft objects, or operate on players or non-Mob
entities.

## Persistence

`is_persistence_required` reads the flag used by vanilla's ordinary distance and inactivity
despawn path. `set_persistence_required` marks the Mob persistent using Minecraft 1.21.1's
`Mob#setPersistenceRequired()`.

Minecraft exposes no public setter to clear this flag: the method only sets it to `true`. Therefore
the SJML API is intentionally one-way, and the `entity.persistence.write` capability grants only
the ability to mark a Mob persistent. This prevents the ordinary despawn branch; it does not block
death, an explicit discard, or a subclass's separate `shouldDespawnInPeaceful()` behavior.

## Item pickup

`can_pick_up_loot` reads Minecraft's `Mob#canPickUpLoot()` flag. `set_can_pick_up_loot` changes it
through `Mob#setCanPickUpLoot(boolean)`. This controls whether the Mob may pick up equipment from
item entities. It does not change loot tables, create drops, or grant direct inventory access.

Reading either flag requires `entity.read`. Persistence writes separately require
`entity.persistence.write`; item-pickup writes separately require `entity.loot_pickup.write`.
Declare the requested capability in the module manifest and grant it on the server.

## ABI

The imports use `sjml_entity_v1` and standard status results:

- `is_persistence_required(handle_ptr, handle_length, output_ptr, output_capacity)`
- `set_persistence_required(handle_ptr, handle_length)`
- `can_pick_up_loot(handle_ptr, handle_length, output_ptr, output_capacity)`
- `set_can_pick_up_loot(handle_ptr, handle_length, enabled)`

Read outputs are little-endian `i32` booleans (`0` or `1`). Write booleans must also be canonical.
Static registration and client-only modules cannot write these server-side Mob flags.
