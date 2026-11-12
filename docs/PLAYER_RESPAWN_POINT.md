# Player respawn points

Server modules can read, replace, or clear an online player's vanilla respawn point:

```rust
use sjml::{player, PlayerRespawnPoint};

let handle = player::online()?.into_iter().next().ok_or_else(|| {
    sjml::SjmlError::new(sjml::ErrorCode::InvalidArgument, "no online player")
})?;

if let Some(current) = player::respawn_point(handle)? {
    // The snapshot contains block coordinates, dimension ID, yaw, and the forced flag.
    player::set_respawn_point(handle, current)?;
}

player::set_respawn_point(
    handle,
    PlayerRespawnPoint::new(12, 80, -6, "minecraft:overworld", 90.0, true)?,
)?;
player::clear_respawn_point(handle)?;
```

Reading requires `entity.read`. Setting and clearing require both `entity.read` and the separate
`player.respawn.write` permission. Calls use a live player handle on the owning logical server
thread; client modules and static registration cannot access player respawn data.

`respawn_point` returns `None` when the player has no bed, respawn anchor, or other configured
vanilla point. A present `PlayerRespawnPoint` contains block X/Y/Z, a canonical dimension ID, yaw
angle, and vanilla's `forced` flag. `clear_respawn_point` resets those fields so Minecraft uses its
normal default spawn after the next death.

The target dimension must already be loaded by the server. The host validates the block coordinate
range, that Y is inside the dimension's build height, and that the block position is inside that
dimension's world border. It does not inspect blocks, test whether a bed or anchor exists, inspect
the target chunk, or request a chunk load. The update stores only the respawn metadata; vanilla may
load the target chunk later when it actually respawns the player. NeoForge spawn hooks can reject a
change, in which case the API returns an error.
