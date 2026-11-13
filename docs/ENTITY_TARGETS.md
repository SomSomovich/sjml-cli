# Loaded Mob targets

The entity API can inspect the current target of a loaded vanilla `Mob`, assign a loaded living
non-player entity, or clear the current target. These calls use handles from the calling module's
own entity table.

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

    if entity::target(mob)? != Some(target) {
        return Err(sjml::SjmlError::new(
            sjml::ErrorCode::InvalidArgument,
            "the selected target was not retained",
        ));
    }
    entity::clear_target(mob)
}
```

`entity::target(mob)` returns `None` both when the Mob has no target and when its current target
is a player. Player handles are not exposed through the entity handle API. `set_target` accepts
only a live, loaded, non-player living entity handle and rejects the Mob itself. Clearing a target
also clears an existing player target.

Every handle must belong to the calling module and identify the same live entity instance on the
same bound logical server. The Mob and selected target must be in the **same loaded dimension**;
the host compares their live server-level identity, not only the dimension's text ID. Calls run
synchronously on the owning server thread. Validation checks the existing entity and chunk state
without requesting or loading chunks. Stale, foreign-owner, unloaded, cross-server, or
cross-dimension handles are rejected.

## ABI v1

The imports are in `sjml_entity_v1` and use 24-byte entity handles:

| Import | Parameters | Required capabilities | Result |
| --- | --- | --- | --- |
| `target` | mob handle pointer and length, output pointer and capacity | `entity.read` | status; output is a versioned optional target handle |
| `set_target` | mob handle pointer and length, target handle pointer and length | `entity.read`, `entity.target.write` | status |
| `clear_target` | mob handle pointer and length | `entity.read`, `entity.target.write` | status |

The target response is little-endian: a 32-bit schema number (`1`), an option tag (`0` for no
exposed target, `1` followed by one 24-byte handle). It is exactly 5 bytes for `None` and 29 bytes
for `Some`; the guest reserves the 29-byte maximum and ignores unused output-buffer padding.
Unknown schemas, invalid tags, malformed handles, and truncated response data fail decoding.
