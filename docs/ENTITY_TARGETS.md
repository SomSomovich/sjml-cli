# Loaded Mob targets

The entity API can inspect a loaded vanilla `Mob`'s current target as a detached snapshot, assign a
loaded living non-player entity, or clear the current target. Assignment uses handles from the
calling module's own entity table; target reads return no handle or Minecraft object.

## Capabilities

Reading a target requires `entity.read`. Assigning or clearing a target requires both
`entity.read` and the separate `entity.target.write` capability. Add each permission to the module
manifest and grant the same capability to that module on the server:

```toml
permissions = ["entity.read", "entity.target.write"]
```

The host only links `set_target` and `clear_target` when both capabilities were requested and
granted. A guest that imports either write function without both grants fails module linking.
Static registration can link these imports but always returns `PERMISSION_DENIED` because it has
no live world access.

## Rust API

```rust
use sjml::{entity, EntityHandle, SjmlResult};

fn choose_target(mob_id: &str, target_id: &str) -> SjmlResult<()> {
    let mut mob: Option<EntityHandle> = None;
    let mut target: Option<EntityHandle> = None;

    for handle in entity::loaded()? {
        let snapshot = entity::snapshot(handle)?;
        if snapshot.uuid == mob_id {
            mob = Some(handle);
        } else if snapshot.uuid == target_id {
            target = Some(handle);
        }
    }

    let mob = mob.expect("the loaded mob must be present");
    let target = target.expect("the loaded target must be present");
    entity::set_target(mob, target)?;

    if entity::target_snapshot(mob)?.as_ref().map(|snapshot| snapshot.uuid.as_str()) != Some(target_id) {
        return Err(sjml::SjmlError::new(
            sjml::ErrorCode::InvalidArgument,
            "the selected target was not retained",
        ));
    }
    entity::clear_target(mob)?;
    if entity::target_snapshot(mob)?.is_some() {
        return Err(sjml::SjmlError::new(
            sjml::ErrorCode::InvalidArgument,
            "the target was not cleared",
        ));
    }
    Ok(())
}
```

`entity::target_snapshot(mob)` returns `Option<EntityTargetSnapshot>` with the target UUID, canonical entity
type and dimension IDs, and bounded coordinates. It returns `None` when the Mob has no target or its
target is a player. The detached snapshot never creates a handle. `entity::target_handle(mob)` keeps
the legacy module-scoped handle read as an explicit alias. The original `entity::target(mob)` also
continues to return `Option<EntityHandle>` using the existing `sjml_entity_v1.target` import; that
handle belongs to the calling module. `set_target` accepts only a live, loaded, non-player living entity
handle, rejects the Mob itself, and bounds target distance to 128 blocks. Clearing a target also
clears an existing player target.

Every handle must belong to the calling module and identify the same live entity instance on the
same bound logical server. The Mob and selected target must be in the **same loaded dimension**;
the host compares their live server-level identity, not only the dimension's text ID. Calls run
synchronously on the owning server thread. Validation checks the existing entity and chunk state
without requesting or loading chunks. Stale, foreign-owner, unloaded, cross-server, or
cross-dimension handles are rejected.

The host checks that the Mob and target are alive and have finite coordinates within ±30,000,000
blocks. It invokes vanilla `Mob.setTarget`; a later `entity::target_snapshot(mob)` call reads back
the actual target selected by Minecraft and its event hooks. Target inspection and mutation never
request or load chunks.

## ABI v1

The imports are in `sjml_entity_v1` and use 24-byte entity handles:

| Import | Parameters | Required capabilities | Result |
| --- | --- | --- | --- |
| `target_snapshot` | mob handle pointer and length, output pointer and capacity | `entity.read` | status; output is a versioned optional detached snapshot |
| `target` | mob handle pointer and length, output pointer and capacity | `entity.read` | status; versioned optional module-scoped handle (existing ABI) |
| `set_target` | mob handle pointer and length, target handle pointer and length | `entity.read`, `entity.target.write` | status |
| `clear_target` | mob handle pointer and length | `entity.read`, `entity.target.write` | status |

`target_snapshot` returns a little-endian schema-v1 optional snapshot containing UUID, type ID,
dimension ID, and three 64-bit coordinates. Its maximum response is 593 bytes; the guest reserves
that bounded size and ignores output-buffer padding. The existing `target` response remains exactly
5 bytes for `None` and 29 bytes for `Some`. Unknown schemas, malformed IDs, invalid option tags,
out-of-range coordinates, and truncated response data fail decoding.
