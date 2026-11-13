# Spawn eggs for registered entities

A module can register a vanilla spawn egg for its own generic mob during
`#[mod_register]`. Declare the entity first, then provide the egg's separate
item path and two 24-bit RGB tint colors:

```rust
use sjml::prelude::*;

#[mod_register]
fn register_content() -> SjmlResult<()> {
    registry::register_entity("river_mite", EntityDefinition::default())?;
    registry::register_entity_spawn_egg(
        "river_mite",
        "river_mite_spawn_egg",
        0xB58D54,
        0x483226,
    )?;
    Ok(())
}
```

The entity and egg must belong to the same module. The host validates both paths,
rejects duplicate item IDs, and limits each color to `0x000000..=0xFFFFFF`.
The egg counts against the normal per-module and total content-registration
quotas. If another mod owns the entity type ID, its egg is skipped.

SJML registers a NeoForge `DeferredSpawnEggItem`, a vanilla `SpawnEggItem`
subclass that keeps the target type bound until the entity registry is ready.
The item uses vanilla spawn-egg interaction behavior, tint layers, and dispenser
behavior, and appears in Minecraft's Spawn Eggs creative tab. The egg item can
also be added to a module-owned creative tab with `registry::add_to_creative_tab`.

The registration requires the module's granted `content.register` capability.
Entity registration completes before eggs are materialized in the item registry;
the import rejects calls outside the synchronous registration callback or after
the relevant registry phase has closed.
