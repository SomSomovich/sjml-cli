# Loaded entity active effects

`entity::active_effects(handle)` returns a detached snapshot of the active status effects on a
loaded living non-player entity. The module must request and receive `entity.read`. The host checks
the entity handle and runs the read on the owning logical-server thread; it never requests a chunk
load.

The response contains at most 64 entries, sorted by canonical lowercase registry ID. Each entry
contains the effect ID, zero-based amplifier, and remaining duration in ticks. A duration of `-1`
means that the vanilla effect is infinite. Effect IDs are limited to 256 UTF-8 bytes and the
versioned response is bounded to 17,416 bytes. If a modded entity exceeds either bound, the whole
read fails instead of returning a partial list.

```rust
use sjml::{entity, EntityHandle, SjmlResult};

fn effects(zombie: EntityHandle) -> SjmlResult<()> {
    for effect in entity::active_effects(zombie)? {
        sjml::log::info(&format!(
            "{} amplifier={} remaining={}", effect.effect_id, effect.amplifier,
            effect.remaining_duration_ticks,
        ));
    }
    Ok(())
}
```

The host returns registry identifiers and scalar values only; Minecraft effect holders and entity
objects do not cross the WASM boundary.
