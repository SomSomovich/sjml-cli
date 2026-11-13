# Player inventory enchantments

The SDK can read, set, or remove one registered enchantment on a non-empty player inventory stack:

```rust
let level = sjml::player_inventory::read_enchantment_level(
    player,
    0,
    "minecraft:sharpness",
)?;

sjml::player_inventory::set_enchantment_level(
    player,
    0,
    "minecraft:sharpness",
    3,
)?;

sjml::player_inventory::remove_enchantment(player, 0, "minecraft:sharpness")?;
```

`read_enchantment_level` returns `None` when the selected item has no such enchantment. All three
operations require a non-empty stack and resolve the canonical enchantment ID from the active
world registry. Unknown IDs, malformed IDs, empty slots, and slot indices outside `0..=40` fail
with `InvalidArgument`.

Set and remove require declared and granted `entity.read` plus `inventory.write`. Reads require
`entity.read` plus `inventory.read`. These are logical-server operations for `server` and `both`
modules; they run synchronously on the owning server thread and resolve a live player session on
each call. Client modules cannot link these imports.

Set levels must be `1..=255` and must also fit the selected enchantment's registered minimum and
maximum. The host checks item applicability and mutually exclusive enchantments before changing
the stack. The update copies the current stack and changes only one entry in its enchantment
component. Enchanted books use stored enchantments. The item, count, durability, custom name,
unrelated components, and other enchantments are preserved. Removing an already absent
enchantment succeeds without changing the stack.

Player inventory indices are `0..=8` for the hotbar, `9..=35` for main storage, `36..=39` for
armor, and `40` for offhand. For ABI function signatures, option encoding, and host error behavior,
see [the ABI reference](ABI.md#enchantments-on-player-inventory-stacks). The dedicated server
GameTest `PlayerInventoryEnchantmentGameTests` exercises read/set/remove, applicability and level
validation, enchanted-book storage, and preservation of unrelated stack data.

## Block inventory stacks

The same bounded operations are available for an existing stack in a loaded
block container:

```rust
let level = sjml::inventory::read_enchantment_level(
    world,
    BlockPos { x: 8, y: 64, z: -4 },
    0,
    "minecraft:sharpness",
)?;

sjml::inventory::set_enchantment_level(
    world,
    BlockPos { x: 8, y: 64, z: -4 },
    0,
    "minecraft:sharpness",
    3,
)?;
sjml::inventory::remove_enchantment(
    world,
    BlockPos { x: 8, y: 64, z: -4 },
    0,
    "minecraft:sharpness",
)?;
```

Reads require `world.read` and `inventory.read`; writes require `world.read`
and `inventory.write`. The host checks the active enchantment registry, level
bounds, item applicability, and conflicts. Enchanted books use stored
enchantments. The stack's item, count, damage, lore, and unrelated components
are preserved. The addressed chunk must already be loaded and the container's
loot table must already be resolved. The host does not add or remove inventory
items; it only replaces the selected stack after verifying the container kept
the same item, count, and components.

The ABI imports are `sjml_inventory_v1.read_block_enchantment_level`,
`set_block_enchantment_level`, and `remove_block_enchantment`. See the
[complete ABI signatures](ABI.md#enchantments-on-block-inventory-stacks).
