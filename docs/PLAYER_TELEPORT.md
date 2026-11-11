# Server-side player teleport

`player::teleport` moves an online player within their current dimension. It
does not accept a dimension ID, so it cannot transfer a player across
dimensions. The host requires a live `PlayerHandle`, validates the owning
server thread and login session, and sends the authoritative teleport through
the server player connection.

The module must request `entity.read` to enumerate/resolve player handles and
`player.teleport` to perform the mutation. Both capabilities must also be
granted by the server administrator:

```toml
[[modules]]
id = "main"
path = "modules/main.wasm"
side = "server"
permissions = ["entity.read", "player.teleport"]
```

```rust
use sjml::prelude::*;

fn move_player(handle: PlayerHandle) -> SjmlResult<()> {
    let destination = player::PlayerTeleport::new(12.5, 70.0, -24.25, 180.0, 0.0)?;
    player::teleport(handle, destination)
}
```

Coordinates must be finite and each axis is limited to ±29,999,984. Yaw must
be finite and within ±36,000 degrees; pitch must be finite and within -90 to
90 degrees. The host additionally enforces the current dimension's build
height and world border. The destination chunk must already be loaded, and the
operation never loads a chunk. Static registration and client execution cannot
teleport players.

## ABI

The host import is `sjml_player_v1.teleport` with signature
`(handle_ptr: i32, handle_len: i32, destination_ptr: i32,
destination_len: i32) -> i32`. The handle is the standard 24-byte player
handle. The destination is exactly 36 bytes: little-endian `u32 schema=1`,
`f64 x`, `f64 y`, `f64 z`, `f32 yaw`, and `f32 pitch`, in that order. Zero
status means success; failures use the stable ABI error code.
