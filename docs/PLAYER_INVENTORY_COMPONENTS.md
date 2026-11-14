# Player Inventory Integer Components

The server inventory API can read and write one integer data component declared by the calling module. It works on the current online player session and uses the module's registered `DataComponentType<Integer>`; guests cannot supply a vanilla component type, JSON, or NBT.

```rust
let charge = sjml::player_inventory::read_integer_component(player, 0, "charge")?;
sjml::player_inventory::write_integer_component(player, 0, "charge", 12)?;
```

Register the component with `registry::register_integer_component("charge", IntegerComponentDefinition::new(minimum, maximum)?)` during module registration. The path is scoped to the package and module. The host resolves only components registered by that same module and validates every written and stored value against the inclusive registered range.

Reads require the declared and administrator-granted `entity.read` and `inventory.read` capabilities. Writes require `entity.read` and `inventory.write`. The imports are available to server and `both` modules, run synchronously on the owning server thread, and resolve the player's current live login session for each call. Client-side modules cannot link them. Handles from another module, closed registries, stale sessions, or players that have left are rejected.

Player slot indices match Minecraft's inventory: `0..=8` hotbar, `9..=35` main inventory, `36..=39` armor, and `40` offhand. Reads return `None` when a slot is empty or the component is unset. Writes require a nonempty stack and an in-range `i32` value. The host copies the stack, changes only the selected registered component, replaces that slot, marks the inventory dirty, and synchronizes the active menu. Item identity, count, damage, custom name, and all other components remain intact.

The ABI imports are `sjml_inventory_v1.read_player_integer_component` and `sjml_inventory_v1.write_player_integer_component`. A read returns its status code and writes a stable optional integer to the guest buffer: one byte `0` for `None`, or tag `1` followed by a little-endian `i32` for `Some(value)`. The write passes the integer directly as an `i32`; permission, module ownership, slot, nonempty-stack, live-session, and registered-range checks are performed by the host.

Boolean components use `registry::register_boolean_component("enabled")` and
`registry::set_item_boolean_component("ruby", "enabled", true)` for declarations
and item defaults. Existing player stacks can be read with
`player_inventory::read_boolean_component(player, slot, "enabled")` and updated
with `write_boolean_component`. They use the same owner, live-session, thread,
and permission checks as integer components. Boolean reads encode `Option<bool>`
as tag `0` for `None`, or tag `1` followed by byte `0` or `1`; writes accept only
those boolean bytes. The host changes only the registered component and
synchronizes the active menu. Block inventory access and the shared data
component contract are documented in [boolean item components](BOOLEAN_ITEM_COMPONENTS.md).

Finite bounded `f32` components use the same module ownership, live-session,
thread, and permission checks. See
[`FLOAT_ITEM_COMPONENTS.md`](FLOAT_ITEM_COMPONENTS.md) for item defaults and
block/player inventory read and write APIs.
