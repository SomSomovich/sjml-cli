# Entity equipment drop chances

SJML exposes the vanilla equipment drop chance for the six fixed slots of a loaded, live, non-player `Mob`:

```rust
use sjml::entity::{self, EntityEquipmentSlot};

let chance = entity::equipment_drop_chance(mob, EntityEquipmentSlot::MainHand)?;
entity::set_equipment_drop_chance(mob, EntityEquipmentSlot::MainHand, 0.25)?;
```

`equipment_drop_chance` requires `entity.read`. The setter requires both `entity.read` and the separate `entity.equipment.drop_chance.write` capability. Reads and writes run synchronously on the owning logical server thread. They validate the live module-owned handle and never create handles or load chunks. Players and non-Mob entities are rejected.

Supported slots are `MainHand`, `OffHand`, `Head`, `Chest`, `Legs`, and `Feet`, matching the fixed slots in [`ENTITY_EQUIPMENT.md`](ENTITY_EQUIPMENT.md). The setter accepts only finite values from `0.0` through `1.0`, inclusive. A vanilla Mob may be configured with a drop chance above `1.0` to preserve an item during death; reading such a value returns `InvalidArgument` because this API intentionally exposes only the normalized range. Values are not clamped.

The setter uses Minecraft 1.21.1's `Mob#setDropChance(EquipmentSlot, float)`. The getter reads the Mob's vanilla hand and armor chance arrays through a narrow host-side accessor; only the selected float crosses the ABI. The versioned request and response are bounded to 36 and 8 bytes respectively.
