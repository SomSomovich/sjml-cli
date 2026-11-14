# Bounded float item components

An SJML module can register a persistent, network-synchronized `f32` data
component with finite inclusive bounds, give its own items a default, and read
or update the value on existing block or player inventory stacks.

```rust
registry::register_float_component(
    "charge",
    registry::FloatComponentDefinition::new(0.0, 100.0)?,
)?;
registry::register_item("capacitor", ItemDefinition::default())?;
registry::set_item_float_component("capacitor", "charge", 25.0)?;

let block_value = inventory::read_float_component(
    world, BlockPos { x: 4, y: 64, z: 2 }, None, 0, "charge",
)?;
inventory::write_float_component(
    world, BlockPos { x: 4, y: 64, z: 2 }, 0, "charge", 42.5,
)?;

let player_value = player_inventory::read_float_component(player, 0, "charge")?;
player_inventory::write_float_component(player, 0, "charge", 42.5)?;
```

Bounds and values must be finite; minimum must not exceed maximum. Values
outside the registered inclusive range, NaN, and either infinity are rejected
by the SDK and checked again by the host. Registration, item defaults, and
runtime access are scoped to the declaring module. Registration and item
defaults require `content.register` during `#[mod_register]`.

Block inventory reads require `world.read` and `inventory.read`; writes require
`world.read` and `inventory.write`. Player inventory reads require
`entity.read` and `inventory.read`; writes require `entity.read` and
`inventory.write`. Runtime calls run on the owning logical-server thread and
never load chunks. Reads return `None` for an absent inventory, empty slot, or
unset component. Writes require an existing non-empty stack, update only the
selected module-owned component, and synchronize the affected inventory.

The component uses Minecraft's data-component persistence and network
synchronization. This API does not expose arbitrary vanilla components or
create arbitrary stacks. See [`BOOLEAN_ITEM_COMPONENTS.md`](BOOLEAN_ITEM_COMPONENTS.md)
for the matching boolean-component contract and
[`PLAYER_INVENTORY_COMPONENTS.md`](PLAYER_INVENTORY_COMPONENTS.md) for player
slot bounds and synchronization details.
