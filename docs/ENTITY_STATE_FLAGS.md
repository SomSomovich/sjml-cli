# Entity state flags

The SDK exposes two Minecraft entity flags for live, loaded non-player entities:

```rust
let currently_glowing = sjml::entity::is_glowing(entity)?;
sjml::entity::set_glowing(entity, true)?;

let currently_silent = sjml::entity::is_silent(entity)?;
sjml::entity::set_silent(entity, false)?;
```

Reading either flag requires `entity.read`. Changing the glowing flag also
requires `entity.glowing.write`; changing the silent flag requires
`entity.state.write`. Both operations run synchronously on the owning logical
server thread and revalidate the module-scoped entity handle. A handle becomes
stale when the entity is removed, leaves its level, or the module closes.

`is_glowing` reads and `set_glowing` changes Minecraft's entity-level glowing
flag. Client tracking and rendering determine whether a connected client shows
the outline; this API does not expose render state. The silent flag controls
Minecraft's ordinary entity sound behavior. These operations do not expose
Java objects or permit arbitrary entity metadata/NBT access.

The API uses the existing `sjml_entity_v1` imports:

- `is_glowing(handle, handle_length, output, capacity)`
- `set_glowing(handle, handle_length, value)`
- `is_silent(handle, handle_length, output, capacity)`
- `set_silent(handle, handle_length, value)`

Boolean outputs are little-endian `u32` values (`0` or `1`); writes accept only
`0` or `1`. Read output spans are checked before Minecraft is accessed.

## Detached movement and visibility snapshot

`entity::state_snapshot(handle)` returns one consistent set of seven read-only
flags: `on_ground`, `in_water`, `on_fire`, `sprinting`, `crouching`,
`invisible`, and `glowing`. It requires `entity.read` and a live loaded
non-player entity handle on the logical-server thread. The schema-v1 response
is exactly 11 bytes: little-endian `u32(1)`, followed in that field order by
seven strict boolean bytes (`0` or `1`). The call does not create handles,
load chunks, or change entity state. The live guest round trip is exercised by
[`EntityStateFlagsGameTests`](../src/main/java/dev/somuch/sjml/neoforge/gametest/EntityStateFlagsGameTests.java).
