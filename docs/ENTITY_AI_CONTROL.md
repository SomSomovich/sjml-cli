# Entity AI control

SJML exposes a narrow AI switch for an already loaded, non-player Minecraft
`Mob`. It does not expose goal selectors, target selection, arbitrary
navigation goals, path inspection, or custom pathfinding hooks. A separate
bounded command can ask vanilla navigation to follow a short path; see
[bounded entity navigation](ENTITY_NAVIGATION.md).

## Permissions and side

Declare both permissions on a server module:

```toml
[mod]
id = "mob-control"
name = "Mob control"
version = "0.1.0"

[runtime]
abi = "^1.32.0"
minecraft = "~1.21.1"
sjml = "^0.42.0"

[[modules]]
id = "main"
path = "modules/main.wasm"
side = "server"
permissions = ["entity.read", "entity.ai.control"]
```

The server administrator must also grant both `entity.read` and
`entity.ai.control`. `entity.read` is needed to obtain and validate an
entity handle. The separate `entity.ai.control` grant is required for both
the AI status read and the write.

## API

`entity::is_ai_enabled(handle)` reads the current vanilla AI flag.
`entity::set_ai_enabled(handle, enabled)` toggles it. For example, disable AI
for loaded zombies:

```rust
use sjml::{entity, SjmlResult};

fn disable_loaded_zombies() -> SjmlResult<()> {
    for handle in entity::loaded()? {
        let snapshot = entity::snapshot(handle)?;
        if snapshot.entity_type_id == "minecraft:zombie" {
            entity::set_ai_enabled(handle, false)?;
        }
    }
    Ok(())
}
```

The host accepts only a current handle for a loaded `Mob` and revalidates it
synchronously on the owning logical-server thread. Players and non-Mob
entities are rejected. Calls from client modules, static registration, a
different thread, or an entity that has unloaded are denied or rejected.

The host changes only Minecraft's Mob no-AI flag; this API does not read, stop,
or create a navigation path. Re-enabling AI resumes normal Mob AI on later
ticks. This call does not request chunks or inspect positions. It operates
only on the already-loaded entity instance identified by the handle.

## ABI bounds

The imports are in `sjml_entity_v1`:

- `is_ai_enabled(handle_ptr, handle_len, output_ptr, output_capacity) -> i32`
  writes one little-endian `u32` value (`0` or `1`) into an output buffer
  of at least four bytes.
- `set_ai_enabled(handle_ptr, handle_len, enabled) -> i32` accepts only
  `enabled = 0` or `1`.

Both calls use the 24-byte ABI v1 entity handle. A malformed handle length,
short or invalid output span, non-boolean flag, stale handle, non-Mob target,
missing grant, or wrong execution context returns an ABI error without
changing entity state.
