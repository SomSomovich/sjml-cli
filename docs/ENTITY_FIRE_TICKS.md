# Entity fire ticks

The SDK exposes bounded reads and writes for the remaining fire duration of a
loaded non-player entity. Both calls are synchronous and run on the owning
logical-server thread.

```rust
use sjml::{entity, SjmlResult};

fn update_fire(entity_handle: sjml::EntityHandle) -> SjmlResult<()> {
    let remaining = entity::fire_ticks(entity_handle)?;
    if remaining > 0 {
        entity::set_fire_ticks(entity_handle, 0)?;
    }
    Ok(())
}
```

The handle can come from `entity::loaded`, `entity::within`, or `entity::raycast`.
It remains scoped to the module and current live entity instance. Players,
client execution, stale handles, and calls off the server thread are rejected.
The host only examines entities already tracked in loaded server levels and
does not load chunks.

Reading requires `entity.read`. Writing also requires the separate
`entity.fire.write` capability and an administrator grant in
`config/sjml/sjml-permissions.toml`:

```toml
[[grant]]
module = "example_pack/main"
capabilities = ["entity.read", "entity.fire.write"]
```

The manifest must request the same capabilities. Fire duration is bounded to
`0..=72_000` ticks; zero clears it. Minecraft stores negative values internally
as a non-burning/fire-immunity sentinel, so reads normalize negative values to
zero. If vanilla or another mod sets a positive duration above the SJML read
bound, the read returns `INVALID_ARGUMENT` instead of truncating it.

The ABI imports are `sjml_entity_v1.fire_ticks(handle_ptr, handle_len,
output_ptr, output_capacity)` and
`sjml_entity_v1.set_fire_ticks(handle_ptr, handle_len, ticks)`. The first writes
one little-endian `u32` to a buffer of at least four bytes. Both return the
standard `i32` ABI status code. Static registration resolves these imports but
denies entity access.
