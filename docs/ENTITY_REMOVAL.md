# Removing a loaded entity

The SDK exposes `entity::discard(handle)` for removing one currently loaded
non-player entity. It requires `entity.read` to obtain and inspect handles and
the separate `entity.remove` capability to perform the mutation. Grant those
capabilities only to modules trusted to remove loaded entities.

```rust
use sjml::prelude::*;

fn remove_one_loaded_entity() -> SjmlResult<()> {
    let Some(handle) = entity::loaded()?.into_iter().next() else {
        return Ok(());
    };
    let snapshot = entity::snapshot(handle)?;
    entity::discard(handle)?;
    host::log(LogLevel::Info, &format!("Removed {}", snapshot.entity_type_id))?;
    Ok(())
}
```

The host validates the owner-scoped entity handle, rejects players and stale
entities, and performs the operation on the logical-server thread. The current
implementation calls Minecraft's entity discard operation: it removes the
entity without running its death behavior or creating drops. It does not load
chunks. On success SJML immediately revokes that module's handle; later use of
the handle reports `STALE_HANDLE`. The guest never receives a Minecraft or JVM
object.

The ABI import is `sjml_entity_v1.discard(handle_ptr, handle_len) -> i32` and
requires an exact 24-byte `EntityHandle`. A static-registration import is linked
only when both permissions are declared and granted, then fails closed with
`PERMISSION_DENIED`.
