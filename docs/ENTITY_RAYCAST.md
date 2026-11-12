# Loaded entity raycasts

Use `entity::raycast` to select the nearest loaded non-player entity whose
Minecraft bounding box intersects a finite segment:

```rust
use sjml::{entity, world, RaycastPoint, SjmlResult, WorldHandle};

fn target(world_handle: WorldHandle) -> SjmlResult<()> {
    let start = RaycastPoint::new(0.5, 64.0, 0.5);
    let end = RaycastPoint::new(0.5, 64.0, 32.5);
    if let Some(entity) = entity::raycast(world_handle, start, end, Some("minecraft:zombie"))? {
        let snapshot = entity::snapshot(entity)?;
        sjml::host::log(sjml::LogLevel::Info, &format!("target: {}", snapshot.entity_type_id))?;
    }
    let _block_hit = world::raycast(
        world_handle,
        start,
        end,
        world::RaycastBlockMode::Collider,
        world::RaycastFluidMode::None,
    )?;
    Ok(())
}
```

The module must request and receive both `entity.read` and `world.read`. The
world handle must be live, module-owned, and belong to the active logical
server. The call is synchronous on the server thread and rejects client,
static-registration, foreign-handle, and stale-world calls.

Endpoints must be finite and within ±30,000,000 blocks. The segment must be
nonzero and at most 128 blocks long. An optional type filter must be a
canonical lowercase namespace:path entity ID. The host inspects entities in
the segment's bounded search box, excludes players and unloaded chunks, and
returns `None` when no matching bounding box intersects. It never loads a
chunk or dimension. If the search box contains more than 4,096 broadphase
entity candidates, the entire query fails before filtering or allocating a
handle. The nearest intersection wins, with canonical UUID order breaking an
exact-distance tie. Only the selected entity receives a module-scoped handle.

This query tests entity geometry only; it does not compare the result with
block occlusion. Use `world::raycast` as well if the caller needs to compare an
entity hit with a block hit. Neither call exposes a Minecraft object to WASM.

The ABI import is `sjml_entity_v1.raycast(request_ptr, request_len,
output_ptr, output_capacity) -> i32`. Schema v1 contains the 24-byte world
handle, two versioned `RaycastPoint` values, and an optional canonical entity
ID. The response is a versioned optional entity handle. Its maximum encoded
response is 29 bytes.
