# Spawning ordinary item stacks

Server modules can spawn a normal Minecraft item stack at a position in a
loaded world:

```rust
use sjml::{world, ItemSpawnVelocity, WorldHandle};

fn drop_reward(world: WorldHandle, x: f64, y: f64, z: f64) -> sjml::SjmlResult<()> {
    world::spawn_item(
        world,
        "minecraft:diamond",
        3,
        world::RaycastPoint::new(x, y, z),
        ItemSpawnVelocity::new(0.0, 0.2, 0.0),
    )
}
```

The module must request and receive both `world.read` and `world.write`.
The identifier can name any item present in the current server item registry,
including a registered item from another mod. `minecraft:air`, unknown IDs,
and non-canonical IDs are rejected. The count must be from 1 through 64 and
must not exceed that item's own stack limit.

The host creates only the item's ordinary default `ItemStack`. This operation
does not accept NBT, custom names, arbitrary data components, or a player
inventory destination. Position values must be finite and within Minecraft's
world coordinate limit. The item bounds must fit inside the dimension's build
height and world border, and every chunk touched by the bounds must already be
loaded. Velocity may be zero; its finite vector magnitude is capped at 4 blocks
per tick. Each module can spawn at most 16 stacks in one server tick. The call
runs on the owning server thread and never requests a chunk load.

The request uses ABI schema 1: the version, module-owned world handle,
length-prefixed canonical item ID, stack count, position, and velocity. Every
field is checked before registry access. Invalid IDs, item-specific stack
counts, out-of-bounds positions, or unloaded chunks return an error; the
per-tick limit returns `SERVICE_BUSY`.
