# Player game mode API

The player SDK can read the current game mode and request a bounded change for an online player. It exposes only the four built-in Minecraft modes, through a detached Rust enum; guest code never receives a Minecraft object.

```rust
use sjml::{player, PlayerGameMode, PlayerHandle, SjmlResult};

fn inspect_and_change(player: PlayerHandle) -> SjmlResult<()> {
    let current = player::game_mode(player)?;
    if current == PlayerGameMode::Survival {
        player::set_game_mode(player, PlayerGameMode::Creative)?;
    }
    Ok(())
}
```

`game_mode(handle)` requires declared and granted `entity.read`. `set_game_mode(handle, mode)` requires both `entity.read` and the separate `player.game_mode.write` capability. Declare both on the server module that calls the setter:

```toml
[[modules]]
id = "main"
path = "modules/main.wasm"
side = "server"
permissions = ["entity.read", "player.game_mode.write"]
```

The module must also receive those permissions from the package's host grants. A missing request or grant prevents the setter import from being linked. The host checks the live player handle and owning logical server thread for every call. It resolves the current object for the same login session, so a handle follows that session through respawn; logout, a stale handle, runtime shutdown, a client execution context, or a non-owning thread cannot change the player.

Client-only modules do not receive the setter import. Modules with `side = "both"` can call the API only from their server instance; their client instance receives `PERMISSION_DENIED`. Static registration resolves declared imports for linking but denies both live reads and changes. The operation changes only the target player's game mode and does not change the world's default mode.

## Stable values and ABI

`PlayerGameMode` has stable `u32` discriminants:

| Value | Rust variant | Minecraft mode |
| ---: | --- | --- |
| 0 | `Survival` | Survival |
| 1 | `Creative` | Creative |
| 2 | `Adventure` | Adventure |
| 3 | `Spectator` | Spectator |

The `sjml_player_v1.game_mode` import accepts `(handle_ptr, handle_len, output_ptr, output_capacity)` and returns an ABI status code. On success it writes exactly 8 bytes: little-endian `u32` schema version `1`, followed by the little-endian `u32` mode value. The SDK decodes this as a versioned `PlayerGameMode` response and rejects unknown mode values.

The `sjml_player_v1.set_game_mode` import accepts `(handle_ptr, handle_len, mode_u32)` and returns an ABI status code. The handle uses the standard 24-byte ABI v1 player-handle encoding. Unknown mode values and malformed handles return `INVALID_ARGUMENT`; missing permissions, client/static contexts, or a wrong server thread return `PERMISSION_DENIED`. Invalid or stale sessions return the corresponding handle error, and a rejected change does not report success.
