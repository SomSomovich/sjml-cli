# Entity teleport

Loaded non-player entities can be moved within their current dimension with
`entity::teleport`:

```rust
entity::teleport(handle, 120.5, 70.0, -24.5)?;
```

The module must request and receive both `entity.read` and `entity.teleport`.
The handle must still identify the same live entity, and the call runs
synchronously on the owning logical-server thread. Players and stale handles
are rejected. The entity must not be riding and must not have passengers.

Each coordinate must be finite and within ±30,000,000. The host checks the
entity's full destination bounding box against the dimension build height and
world border. Every chunk touched by that box must already be loaded;
teleportation never requests chunks or creates chunk tickets. The operation
uses Minecraft's same-dimension `Entity.teleportTo` path and confirms the
entity remains at the requested position in its original loaded level.

Declare the capabilities in the module manifest and grant both to the module
in `config/sjml-permissions.toml`:

```toml
permissions = ["entity.read", "entity.teleport"]
```

```toml
[[grant]]
module = "mymod/main"
capabilities = ["entity.read", "entity.teleport"]
```

The host import is `sjml_entity_v1.teleport(request_ptr, request_len) -> i32`.
Its request is exactly 52 bytes: little-endian `u32 schema=1`, the 24-byte ABI
entity handle, then `f64 x`, `f64 y`, and `f64 z`. Native SDK builds report the
host API as unsupported; static registration cannot teleport live entities.
