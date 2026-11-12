# Module-owned integer item components

SJML modules can register bounded integer data components, assign item defaults, and read or write
the value on an existing stack in a loaded block inventory. These values are persistent and
network-synchronized through Minecraft's data-component system. The API exposes no arbitrary NBT or
generic Java component access.

```rust
use sjml::{inventory::{self, BlockPos}, registry::{self, IntegerComponentDefinition}};

registry::register_integer_component(
    "charge",
    IntegerComponentDefinition::new(0, 100)?,
)?;
registry::set_item_integer_component("ruby", "charge", 25)?;

let current = inventory::read_integer_component(
    world, BlockPos { x: 4, y: 64, z: 2 }, None, 0, "charge",
)?;
inventory::write_integer_component(
    world, BlockPos { x: 4, y: 64, z: 2 }, 0, "charge", 30,
)?;
```

Registration and item defaults require `content.register` during `#[mod_register]`. Component paths
are module-local, and the host permits a module to access only components that it registered under
its own package and module ID. Defaults must refer to an item declared by that same module.

Inventory reads require `world.read` and `inventory.read`; writes require `world.read` and
`inventory.write`. Integer inventory imports are available to `server` and `both` modules, never
to `client` modules. Calls run synchronously on the owning server thread and validate the world
handle before looking up an item. Reads use the optional sided item-handler capability and return
`None` for an absent inventory, empty slot, or unset component. Writes are unsided because they
update a block entity implementing Minecraft `Container`; they require a non-empty slot and copy
the stack before replacing its component. The operation rejects unloaded chunks without loading
them, invalid slots, unregistered or foreign component paths, and values outside the registered
inclusive `minimum..=maximum` range. Unrelated item data and the stack count are preserved.

The `inventory.read` and `inventory.write` permissions are separate. The host's persistent codec
and network codec both enforce the registered range, and a write rechecks it before changing the
stack. The string component APIs retain their existing behavior.

For module-owned boolean item data, see [boolean item components](BOOLEAN_ITEM_COMPONENTS.md).
