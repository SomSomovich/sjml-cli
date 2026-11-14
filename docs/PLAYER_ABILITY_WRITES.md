# Player ability writes

The player API can change a live server player's vanilla `Abilities` fields. Every write requires
both the module permission and server grant for `entity.read` and `player.abilities.write`:

```toml
permissions = ["entity.read", "player.abilities.write"]
```

The API is server-side and checks the current login session and owning server thread on every call.
Client modules and static registration cannot perform these writes. Real changes are synchronized
to the player through Minecraft's `ServerPlayer.onUpdateAbilities()`.

## Setters

```rust
player::set_may_fly(player, true)?;
player::set_flying(player, true)?;
player::set_instabuild(player, true)?;
player::set_invulnerable(player, false)?;
player::set_may_build(player, true)?;
player::set_flying_speed(player, 0.05)?;
player::set_walking_speed(player, 0.1)?;
```

| Function | Minecraft field | Behavior |
| --- | --- | --- |
| `set_may_fly(handle, enabled)` | `mayfly` | Disabling flight permission also clears current flight in the same server-thread update. |
| `set_flying(handle, enabled)` | `flying` | Enabling flight is rejected unless `mayfly` is already true; disabling flight is allowed. |
| `set_instabuild(handle, enabled)` | `instabuild` | Changes instant-build ability without changing game mode. |
| `set_invulnerable(handle, enabled)` | `invulnerable` | Changes the vanilla player invulnerability ability. |
| `set_may_build(handle, enabled)` | `mayBuild` | Changes the vanilla ability to build and interact with blocks. |
| `set_flying_speed(handle, speed)` | flight speed | Sets the vanilla ability flight-speed scalar. |
| `set_walking_speed(handle, speed)` | walking speed | Sets the vanilla ability walking-speed scalar. |

Both speed setters accept finite values in `0.0..=0.5`. Vanilla's ordinary defaults are about
`0.05` for flight and `0.1` for walking. The SDK validates the range before calling the host, and
the host independently validates it again.

These functions mutate only the named abilities, except for clearing `flying` when `mayfly` is
disabled to preserve Minecraft's `flying => mayfly` invariant. They do not change game mode. The
`player::abilities(handle)` snapshot remains schema v1 and contains only `may_fly`, `flying`,
`invulnerable`, and `may_build`; it does not include `instabuild` or speed values.

See [player abilities](PLAYER_ABILITIES.md) for the read snapshot contract.
