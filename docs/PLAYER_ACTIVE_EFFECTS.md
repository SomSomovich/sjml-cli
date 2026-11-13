# Player active effects

The Rust SDK exposes `player::active_effects(handle)` to read the active status
effects on a player's current live server session. The module must request and
receive `entity.read`. The call runs on the logical-server thread and resolves
the handle against the current login session on every call. A player handle
continues to refer to the session through respawn and dimension changes, but
logout and runtime shutdown invalidate access. Client modules and static
registration cannot read live player effects.

```rust
let effects = sjml::player::active_effects(player_handle)?;
for effect in effects {
    sjml::log::info(&format!(
        "{} amplifier {} for {} ticks",
        effect.effect_id, effect.amplifier, effect.remaining_duration_ticks
    ));
}
```

The response is detached data. It contains no player, level, effect-instance,
or other Minecraft object references. Effects are sorted by canonical
lowercase `namespace:path` ID and limited to 64 records. Each ID is at most 256
UTF-8 bytes, amplifiers are `0..=255`, and remaining duration is nonnegative
ticks or `-1` for an infinite-duration effect. The host rejects an oversized
effect list rather than returning a partial snapshot.

The manifest permission is:

```toml
[[modules]]
side = "server"
permissions = ["entity.read"]
```

The ABI import is `sjml_player_v1.active_effects(handle_ptr, handle_len,
output_ptr, output_capacity) -> status`. It uses the same bounded schema-v1
record as `entity::active_effects`: a little-endian u32 schema and count,
followed by schema-v1 effect records with a length-prefixed ID, u32 amplifier,
and i32 remaining duration. The maximum response buffer is 17,416 bytes.
