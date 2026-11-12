# Loaded entity spatial queries

Use `entity::within` to select loaded non-player entities around a world position:

```rust
use sjml::{entity, BlockPos, EntityHandle, SjmlResult, WorldHandle};

fn targets(world: WorldHandle, center: BlockPos) -> SjmlResult<Vec<EntityHandle>> {
    entity::within(world, center, 12.0, 4.0, Some("minecraft:zombie"))
}
```

The module must request and receive both `entity.read` and `world.read`. The supplied `WorldHandle` must belong to the calling module and its active logical server. The call runs synchronously on that server's owning thread; client modules, static registration, stale worlds, foreign handles, and missing capability grants are denied.

The horizontal radius measures distance in the X/Z plane, producing a cylinder around `center`; the vertical radius is an independent absolute Y distance. Both radii must be finite and between 0 and 64 blocks. The optional type filter must be a canonical lowercase namespace:path entity ID. Results contain at most 256 handles. If more entities match, the complete query fails with `PayloadTooLarge`; the host returns no partial selection and allocates no new handles.

The provider examines entities already tracked by the selected `ServerLevel` and verifies each candidate remains in a loaded chunk. It never requests a chunk or loads a dimension. Matching handles are sorted by canonical UUID, reused for the same live entity instance, and retain the usual module ownership, generation, world, and live-entity checks. Players are excluded; use the separate player API for player targets.

The ABI uses `sjml_entity_v1.within(request_ptr, request_len, output_ptr, output_capacity) -> i32`. The schema-v1 request contains a world handle, schema-v1 `BlockPos`, two little-endian `f64` radii, and an optional length-prefixed entity type ID. The response contains schema version, count, and up to 256 24-byte entity handles. The request is limited to 321 bytes and the response to 6,152 bytes.
