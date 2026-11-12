# Cross-dimension player teleport

Use `player::teleport_to_world` when a server module needs to move a live player
session to a selected loaded dimension (including the current one):

```rust
use sjml::{player, world, ErrorCode, PlayerHandle, PlayerTeleport, SjmlError, SjmlResult};

fn send_to_nether(player_handle: PlayerHandle) -> SjmlResult<()> {
    let Some(nether) = world::get("minecraft:the_nether")? else {
        return Err(SjmlError::new(ErrorCode::InvalidArgument, "the Nether is not loaded"));
    };
    player::teleport_to_world(
        player_handle,
        nether,
        PlayerTeleport::new(0.5, 64.0, 0.5, 90.0, 0.0)?,
    )
}
```

The module must request and receive `entity.read`, `player.teleport`, and
`world.read`. The player handle must identify this module's current online
login session, and the target world handle must belong to this module and its
active logical server. Client contexts and static registration are denied.
The operation runs on the owning server thread.

The host checks finite bounded coordinates and rotation, target dimension build
height and world border, and that the destination chunk is already loaded. It
rejects an unavailable or stale target world and never requests a chunk load.
The server's dimension-travel hook can cancel the transfer, which is returned
as a permission error. A successful transfer retains the login-session handle.

The ABI import is
`sjml_player_v1.teleport_to_world(player_ptr, player_len, world_ptr, world_len,
destination_ptr, destination_len) -> i32`. It reuses schema-v1
`PlayerTeleport` as its absolute destination. In Minecraft 1.21.1, the host
uses `ServerPlayer.teleportTo(ServerLevel, x, y, z, yaw, pitch)`, which performs
the `DimensionTransition` and dimension-change packet flow. The overload that
accepts relative movement flags adds a `POST_TELEPORT` chunk ticket, so this
operation does not use it. `PlayerList.respawn` replaces the `ServerPlayer` and
is reserved for respawn, so it is not used for ordinary travel.
