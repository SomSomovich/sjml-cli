# Removing status effects from entities

Server modules can remove one registered status effect from a loaded living
non-player entity with `sjml::entity::remove_effect`:

```rust
use sjml::{entity, SjmlResult};

fn clear_slowness(target: entity::EntityHandle) -> SjmlResult<()> {
    entity::remove_effect(target, "minecraft:slowness")
}
```

The module must request `entity.read` and `entity.effect.remove` in
`sjml.toml`, and the server administrator must grant both capabilities in
`config/sjml-permissions.toml`. The host validates the module-owned handle,
live loaded target, logical-server context, and server thread on every call.
Players, nonliving entities, stale handles, client modules, and registration-
phase calls are rejected. The guest receives only a status result, never a
Minecraft object.

Effect identifiers use canonical lowercase `namespace:path` syntax and must
refer to a registered status effect. An effect that is not currently active
is a successful no-op. `entity.effect.remove` is independent from
`entity.effect.apply`, so a module that only removes effects does not need to
request a capability that can apply them.

The Java import is `sjml_entity_v1.remove_effect(handle_ptr, handle_len,
effect_id_ptr, effect_id_len) -> status`. It reuses the bounded UTF-8 identifier
and 24-byte entity-handle encodings described in [ABI.md](ABI.md).
