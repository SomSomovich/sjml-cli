# Entity state writes

The SDK can change the vanilla gravity and invulnerability flags on a live, loaded non-player
entity:

```rust
entity::set_no_gravity(handle, true)?;
entity::set_invulnerable(handle, true)?;
```

Both operations are synchronous on the owning logical-server thread. They resolve an existing
module-owned entity handle and recheck its generation and live instance. They do not allocate
handles, load chunks, or pass Minecraft objects to guest code.

## Capabilities

Both calls require `entity.read` in addition to their write capability:

| SDK call | Additional capability |
| --- | --- |
| `entity::set_no_gravity` | `entity.state.write` |
| `entity::set_invulnerable` | `entity.invulnerability.write` |

The capabilities are independent. `entity.state.write` also continues to authorize
`entity::set_silent`; `entity.invulnerability.write` affects only `entity::set_invulnerable`.
Declare each requested permission in the module manifest and grant it to that module on the server.

## Behavior and imports

`set_no_gravity` calls Minecraft 1.21.1's `Entity#setNoGravity(boolean)`. It changes the entity's
ordinary no-gravity flag. `set_invulnerable` calls `Entity#setInvulnerable(boolean)`. Minecraft's
normal damage pipeline still determines which damage sources bypass invulnerability.

Both imports use `sjml_entity_v1` and accept `handle_ptr`, `handle_length`, and a canonical `i32`
boolean value (`0` or `1`):

- `set_no_gravity`
- `set_invulnerable`

The calls reject malformed booleans and are denied during static registration, on the logical
client, off the owning server thread, or for stale, foreign, or non-entity handles.
