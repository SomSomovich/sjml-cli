# Entity rotation and head yaw

The entity API exposes detached rotation snapshots for loaded, non-player entities. The original
`entity::rotation` / `entity::set_rotation` calls remain available for yaw and pitch. The
`entity::orientation` / `entity::set_orientation` calls also read or set head yaw independently:

```rust
use sjml::prelude::*;

fn rotate_entity(handle: EntityHandle) -> SjmlResult<()> {
    let original = entity::orientation(handle)?;
    let desired = EntityOrientation::new(90.0, -20.0, 120.0)?;
    entity::set_orientation(handle, desired)?;
    entity::set_orientation(handle, original)
}
```

Both reads require `entity.read`. Writes require `entity.read` and the separate
`entity.rotation.write` grant. Yaw and head yaw must be finite and within ±36,000 degrees; pitch
must be finite within -90..=90 degrees. The SDK constructors validate these bounds, and the host
checks them again.

Every call checks the module-owned entity handle, the live entity instance, logical-server context,
and owning server thread. The provider accepts only entities in already-loaded chunks and never
requests or loads a chunk. Player handles use the separate `player` API. Client modules and static
registration cannot access these imports.

The compiled SDK example is
[`entity_orientation.rs`](../sjml-cli/sjml-sdk/examples/entity_orientation.rs). The dedicated
server `EntityRotationGameTests` fixture reads, changes, and restores yaw, pitch, and head yaw on a
loaded pig.

## ABI v1

The existing `rotation` / `set_rotation` imports keep their 12-byte schema-v1 value:
`u32(1)`, `f32 yaw`, `f32 pitch`.

The `orientation` import writes a 16-byte schema-v1 snapshot. `set_orientation` accepts the same
16-byte request alongside the existing 24-byte entity handle:

1. `u32(1)` schema version.
2. `f32` yaw.
3. `f32` pitch.
4. `f32` head yaw.

All fields are little-endian. An invalid value or wrong payload length returns `INVALID_ARGUMENT`;
an unsupported schema returns `INCOMPATIBLE_ABI_VERSION`. Output-buffer size is checked before
resolving the live entity handle.
