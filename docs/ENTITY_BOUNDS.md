# Entity bounds

`entity::bounds(handle)` reads the current axis-aligned bounding box (AABB) of a live, loaded,
non-player entity and returns detached world-space coordinates:

```rust
let bounds = entity::bounds(handle)?;
let width = bounds.max_x - bounds.min_x;
```

The result is `EntityBounds` with `min_x`, `min_y`, `min_z`, `max_x`, `max_y`, and `max_z` fields.
The host reports Minecraft's current box; coordinates are finite, limited to ±30,000,064 blocks,
and ordered so each minimum is no greater than its corresponding maximum.

## Access and lifetime

The module must declare and receive the `entity.read` capability. The call runs synchronously on
the owning logical-server thread and checks the handle's module owner, entity kind, generation, and
live entity instance. It is unavailable in client modules. Obtain an entity handle through
`entity::loaded()`, `entity::within(...)`, or `entity::raycast(...)` first. This read does not create
handles, load chunks, or access an entity whose chunk is not already loaded. The returned value is
detached; it does not keep the entity alive or update after the call.

The host import is `sjml_entity_v1.bounds(handle_ptr, handle_length, output_ptr, output_capacity)`.
Its schema-v1 response is exactly 52 bytes: a little-endian `u32` schema version (`1`), followed by
six little-endian `f64` values in the field order above. A short output buffer is rejected before
the host reads the entity.

## Errors

- `PermissionDenied`: `entity.read` is missing, execution is on the wrong side or thread, or a
  static-registration call tries to access a live entity.
- `StaleHandle`: the entity was removed, unloaded, or its chunk is no longer loaded, or the handle
  belongs to another module or entity generation.
- `HandleTableClosed` / `HandleKindMismatch`: the module is closed or the supplied handle is not an
  entity handle.
- `PayloadTooLarge`: the guest output buffer cannot hold the fixed response.
- `InvalidArgument`: the provider returned non-finite, out-of-range, or reversed bounds.
