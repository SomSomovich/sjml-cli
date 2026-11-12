# Module-owned boolean item components

An SJML module can register one of its own persistent, network-synchronized
boolean data components, assign defaults to items it declares, and read or
change that value on existing block or player inventory stacks.

```rust
registry::register_boolean_component("activated")?;
registry::set_item_boolean_component("signal_token", "activated", true)?;

let block_value = inventory::read_boolean_component(
    world, BlockPos { x: 4, y: 64, z: 2 }, None, 0, "activated",
)?;
inventory::write_boolean_component(
    world, BlockPos { x: 4, y: 64, z: 2 }, 0, "activated", false,
)?;

let player_value = player_inventory::read_boolean_component(player, 0, "activated")?;
player_inventory::write_boolean_component(player, 0, "activated", true)?;
```

Registration and defaults require `content.register` during `#[mod_register]`.
The item, component type, and runtime caller must belong to the same package
module. Defaults apply to the default stack for an item or block item declared
by that module. A boolean component path contains 1–256 lowercase ASCII
letters, digits, `.`, `_`, or `-`.

Block inventory reads require `world.read` plus `inventory.read`; writes require
`world.read` plus `inventory.write`. Player inventory reads require
`entity.read` plus `inventory.read`; writes require `entity.read` plus
`inventory.write`. Runtime calls are server-side and run on the owning server
thread. Block reads use the optional sided item-handler capability. Block
writes require a non-empty slot in a loaded block `Container`; player writes
require a non-empty live player slot. Both write paths copy the existing stack,
change only this module's component, preserve the remaining stack data, and
synchronize the relevant inventory. No chunk is loaded by these APIs.

Reads return `None` for an absent inventory or an empty slot and return
`Some(false)` distinctly from `None`. The ABI represents `Option<bool>` with a
one-byte tag (`0` for `None`, `1` for `Some`) followed by a strict boolean byte
(`0` or `1`) when present. Writes also accept only `0` or `1` at the host ABI
boundary. Vanilla components and NBT are not exposed by this API.
