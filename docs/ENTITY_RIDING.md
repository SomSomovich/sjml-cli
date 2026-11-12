# Entity riding

A server module with `entity.read` can read detached vehicle and passenger identities for one
loaded non-player entity. To ask Minecraft to mount or dismount entities, also request and receive
`entity.mount.write`:

```toml
permissions = ["entity.read", "entity.mount.write"]
```

```rust
use sjml::{entity, host::{self, LogLevel}};

let relationships = entity::riding_snapshot(entity_handle)?;
if let Some(vehicle) = relationships.vehicle {
    host::log(LogLevel::Info, &format!("vehicle {} ({})", vehicle.uuid, vehicle.entity_type_id))?;
}
for passenger in relationships.passengers {
    host::log(LogLevel::Info, &format!("passenger {} player={}", passenger.uuid, passenger.is_player))?;
}

// Both handles must be current non-player handles from the same loaded world.
let mounted = entity::start_riding(passenger_handle, vehicle_handle)?;
let dismounted = entity::stop_riding(passenger_handle)?;
```

`riding_snapshot` returns a vehicle identity (or none) and at most 16 passenger identities sorted
by canonical UUID. Each identity contains a UUID, a registered entity type ID, and an `is_player`
flag. Player relationships can be observed, but this API does not issue player handles or expose
player state.

Mount changes run on the owning logical server thread. The passenger must not already have a
vehicle; SJML does not silently move it from another mount. `start_riding` calls Minecraft with
`force=false` and returns false when vanilla refuses the mount. Both handles are revalidated and
must identify live non-player entities in the same loaded world. `stop_riding` returns false when
the passenger has no vehicle. Neither operation requests or loads chunks. Cross-world mounting,
self-mounting, player handles, and forced mounts are unsupported.

The Wasm import module is `sjml_entity_v1`. `riding_snapshot` uses the schema-v1 stable encoding
of `EntityRidingSnapshot`; `start_riding` and `stop_riding` write a schema-compatible `u32` result
value of 0 or 1 to guest memory. Mutation imports require both `entity.read` and
`entity.mount.write`.
