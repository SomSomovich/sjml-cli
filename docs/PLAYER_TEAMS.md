# Live player scoreboard teams

The Rust SDK can read or change a live server player's membership in an
existing scoreboard team:

```rust
let current = sjml::player::team(player_handle)?;
sjml::player::set_team(player_handle, "red")?;
sjml::player::clear_team(player_handle)?;
```

The read requires `entity.read`. Both changes require `entity.read` and the
separate `player.team.write` capability:

```toml
[[modules]]
side = "server"
permissions = ["entity.read", "player.team.write"]
```

The host resolves the player handle to its current live login session and
checks the owning server thread, `ServerPlayer`, and active `ServerLevel` on
each call. Client modules, stale sessions, and static registration cannot
change membership. The returned value is a detached `Option<String>` team ID;
it contains no Minecraft references.

`set_team` accepts a 1–64 byte ASCII team ID made of letters, digits, `.`, `_`,
`+`, or `-`. The named team must already exist on the current server
scoreboard. The call can move the player from their current team, including a
vanilla team or a team owned by another module. `clear_team` removes the
current assignment and succeeds if the player is already unassigned. The API
does not create or remove teams; use the owner-scoped team declarations in
[`SCOREBOARD.md`](SCOREBOARD.md) for teams owned by the calling module.

The ABI imports are `sjml_player_v1.team`, `set_team`, and `clear_team`.
`team` writes a schema-v1 optional string into a 73-byte bounded buffer: a
little-endian u32 schema, a `0` tag for no assignment, or a `1` tag followed by
a little-endian u32 byte length and the ASCII name. Both write imports return
status only.

The guest-driven dedicated-server check is
[`PlayerTeamGameTests`](../src/main/java/dev/somuch/sjml/neoforge/gametest/PlayerTeamGameTests.java).
