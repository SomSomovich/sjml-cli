# Guest particle visuals

SJML guest modules can register simple particle types and spawn them through
the bounded world API. `registry::register_particle(path)` keeps the shared
vanilla `minecraft:generic_0` sprite. To use package art, declare a
`ParticleDefinition` during `#[mod_register]`:

```rust
use sjml::prelude::*;

#[mod_register]
fn register_content() -> SjmlResult<()> {
    let particle = registry::ParticleDefinition::new("single_demo:particle/ruby_spark")?
        .with_scale(0.75)?;
    registry::register_particle_with_definition("ruby_spark", particle)
}
```

For the texture ID `single_demo:particle/ruby_spark`, the package must contain
`assets/single_demo/textures/particle/ruby_spark.png`. The host requires the
texture namespace to match the package ID and accepts only safe `particle/`
atlas resource paths owned by that package. The generated particle description
refers to the declared sprite, while the validated package-resource pack mounts
the PNG into Minecraft's particle atlas.

The base scale is a finite multiplier from `0.05` through `4.0`. Particle motion,
20-tick lifetime, zero gravity, no-physics behavior, and translucent render type
remain the shared vanilla-safe defaults; guests cannot currently choose custom
motion or lifetime. Spawning still requires `world.read` and `world.write`, a
loaded server world, a position inside build height, and an already loaded
chunk. It never requests a chunk load.

Particle types are bounded by the same per-module and total content-registration
quotas as other static content. Invalid texture ownership, path, or scale values
reject the declaration before it is staged. Package rollback removes the type,
sprite, and scale together.
