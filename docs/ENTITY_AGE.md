# Ageable Mob age API

The entity API exposes bounded age reads and writes for loaded, living, non-player Minecraft
`AgeableMob` instances such as pigs, cows, villagers, and horses. It does not apply to arbitrary
Mobs such as zombies or to players.

```rust
use sjml::prelude::*;

fn inspect_and_grow(mob: EntityHandle) -> SjmlResult<()> {
    let age = entity::age_ticks(mob)?;
    if entity::is_baby(mob)? {
        entity::set_baby(mob, false)?;
    } else {
        entity::set_age_ticks(mob, age)?;
    }
    Ok(())
}
```

Reads require `entity.read`. Writes also require the separately granted `entity.age.write`
capability and are available only to a logical-server module on the owning server thread. The host
resolves the existing module-owned entity handle and never loads chunks or returns Minecraft
objects.

Age is bounded to `-24,000..=24,000` ticks. Negative values represent babies, zero represents adult
age, and positive values represent vanilla post-breeding cooldown. `set_baby(true)` selects the
youngest supported baby age; `set_baby(false)` sets age to zero. Use `set_age_ticks` for a precise
age or cooldown. Minecraft advances age using its normal entity tick behavior after the change.

The guest ABI uses schema-v1 8-byte little-endian responses for age and baby-state reads. The
`set_age_ticks` import carries a signed 32-bit age, and `set_baby` accepts only the canonical 0/1
boolean encoding.

```toml
[[modules]]
id = "main"
path = "modules/main.wasm"
side = "server"
permissions = ["entity.read", "entity.age.write"]
```
