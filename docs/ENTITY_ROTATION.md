# Entity rotation

The Rust SDK reads and changes yaw and pitch on one live, loaded non-player entity:

```rust
let current = sjml::entity::rotation(handle)?;
let desired = sjml::entity::EntityRotation::new(90.0, -20.0)?;
sjml::entity::set_rotation(handle, desired)?;
```

Read requires `entity.read`. A write requires both `entity.read` and
`entity.rotation.write`. Angles use degrees; yaw must be finite and within
±36,000 degrees, and pitch must be finite within -90..=90 degrees. `EntityRotation::new`
checks these bounds before the host call, and the host repeats the checks.

The host accepts only an entity handle owned by the current module that still
identifies the same loaded non-player entity. Calls run on the owning logical
server thread. The provider checks the entity's current server level, UUID
mapping, and already-loaded chunk before reading `Entity.getYRot()` /
`Entity.getXRot()` or writing `Entity.setYRot()` / `Entity.setXRot()`. It never
requests or loads a chunk. Player handles use the separate `player` API.

The `sjml_entity_v1.rotation` import writes a 12-byte schema-v1 value:
little-endian `u32(1)`, `f32 yaw`, then `f32 pitch`. `set_rotation` accepts the
same exact 12-byte request alongside a 24-byte entity handle. An unsupported
schema or wrong request length returns `INVALID_ARGUMENT`; a short read buffer
returns `PAYLOAD_TOO_LARGE` before the entity is queried. Static registration
and client-side calls are denied.

`EntityRotationImportsTest` checks the wire encoding, bounds, permissions,
thread validation, output preflight, and stale-handle rejection. The dedicated
server `EntityRotationGameTests` fixture uses the guest command
`entity.rotation.check <entity_id>` to read a pig's initial rotation, set and
read a temporary rotation, and restore the original values.
