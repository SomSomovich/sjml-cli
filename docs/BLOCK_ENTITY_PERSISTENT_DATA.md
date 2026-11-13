# Persistent data on registered block entities

Modules can store a small typed record on each block entity created for a block
entity type registered by that same module. Values are part of Minecraft's
ordinary block-entity save data and survive world saves and reloads. The API
does not expose a mutable `CompoundTag` or permit writing data to vanilla or
another module's block entities.

```rust
use sjml::prelude::*;

fn update_machine(world: WorldHandle, position: BlockPos) -> SjmlResult<()> {
    let machine = world
        .block_entity(position)?
        .ok_or_else(|| SjmlError::new(ErrorCode::InvalidArgument, "machine is not loaded"))?;

    machine.set_data("active", PersistentDataValue::Boolean(true))?;
    machine.set_data("charge", PersistentDataValue::Integer(7))?;
    machine.set_data("label", PersistentDataValue::String("north generator".into()))?;

    let charge = machine.get_data("charge")?;
    assert_eq!(charge, Some(PersistentDataValue::Integer(7)));
    machine.remove_data("label")?;
    Ok(())
}
```

Getting a handle uses the existing loaded-only `world::block_entity` API and
requires `world.read`. Typed reads also require `block_entity.data.read`. Typed
writes and removals require `block_entity.data.read`, `block_entity.data.write`,
and `world.write`. The host verifies that the block entity is still live and
that its registered owner matches both the package and module making the call.
Unloaded or replaced entities produce a stale-handle error; the API never loads
a chunk. Writes run only on the owning logical-server thread.

Each entity accepts at most 64 keys. Keys contain 1–64 lowercase ASCII letters,
digits, or underscores. Values are Booleans, signed 32-bit integers, or
printable UTF-8 strings up to 2,048 bytes. The combined key/value data on one
entity is capped at 16 KiB. Each module can make up to 64 write/removal
attempts per logical-server tick, shared across dimensions. The
host drops invalid or over-limit saved values while loading so malformed data
cannot prevent a block entity from loading.

This API is for small per-block state. Use the separate module storage API for
larger records or data that must outlive the block entity; use bounded NBT
snapshots only when `block_entity.nbt.read` is needed to inspect existing tags.
SJML omits its private persistent-data compound from generic NBT snapshots so
that the raw-tag API does not bypass owner and data permissions.
