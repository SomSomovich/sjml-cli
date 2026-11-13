# Bounded vanilla projectile launches

Server modules can launch a vanilla snowball or egg from an explicit loaded
world position with a validated velocity. The API exposes no arbitrary entity
IDs or Minecraft objects.

```rust
use sjml::prelude::*;

fn toss(world: WorldHandle, position: RaycastPoint) -> SjmlResult<()> {
    world::spawn_projectile(
        world,
        VanillaProjectileKind::Snowball,
        position,
        ProjectileVelocity::new(1.25, 0.5, 0.0),
    )
}
```

The module must request `world.read` and `projectile.spawn`, and the server
administrator must grant both. `world.read` supplies a module-scoped handle to
an already loaded dimension. Spawning runs synchronously on that server's main
thread. The position must be finite, inside the world's build height and border,
and every intersected chunk must already be loaded. The API never requests a
chunk load.

Only `VanillaProjectileKind::Snowball` and `VanillaProjectileKind::Egg` are
accepted. Launch velocity components must be finite, and speed must be between
0.1 and 4.0 blocks per tick. A module may spawn at most four projectiles per
server tick, shared across dimensions. Vanilla collision behavior is retained:
snowballs deal their ordinary 3 damage to Blazes, and eggs retain their vanilla
chance to hatch chickens.

```toml
[[modules]]
id = "main"
path = "modules/main.wasm"
side = "server"
permissions = ["world.read", "projectile.spawn"]
```
