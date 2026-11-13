# Entity vitals and healing

The Rust SDK can read bounded vitals and request an ordinary vanilla heal for a loaded living
non-player entity. The host keeps the Minecraft entity object inside Java and handles both calls
on the owning logical-server thread.

Request `entity.read` to enumerate entities and call `entity::vitals`. Add
`entity.health.write` to call `entity::heal`:

```toml
[[modules]]
id = "main"
path = "modules/main.wasm"
side = "server"
permissions = ["entity.read", "entity.health.write"]
```

```rust
use sjml::{entity, EntityHandle, SjmlResult};

fn restore_some_health(handle: EntityHandle) -> SjmlResult<()> {
    let before = entity::vitals(handle)?;
    let missing = before.max_health - before.health;
    if missing > 0.0 {
        entity::heal(handle, missing.min(4.0))?;
    }
    let after = entity::vitals(handle)?;
    // A heal event callback or vanilla behavior may reject the request.
    let _health_now = after.health;
    Ok(())
}
```

`EntityVitals` reports `health`, `max_health`, and `absorption` as finite `f32` values. The exact
schema-v1 ABI response is 16 bytes: little-endian `u32(1)`, current health, effective maximum
health, then absorption. Health must be in `0..=max_health`; maximum health must be positive and
at most 1,000,000; absorption must be in `0..=1,000,000`.

`entity::heal(handle, amount)` accepts a finite amount in `(0, 64]`. It calls Minecraft's normal
`LivingEntity.heal` method. Existing `entity.heal_attempt` listeners and vanilla behavior remain
authoritative, so a successful API result means the request ran; it does not guarantee health
changed. The SDK does not provide an arbitrary health setter for non-player entities.

Both calls require a live handle for a non-player living entity already present in a loaded chunk.
They reject client context, stale handles, nonliving targets, and calls off the owning server
thread. They never load chunks. A read requires `entity.read`; healing requires both `entity.read`
and `entity.health.write`.
