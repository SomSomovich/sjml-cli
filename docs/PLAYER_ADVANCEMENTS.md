# Player advancements

Server modules can grant or revoke progress on a loaded Minecraft advancement for a live online player:

```rust
use sjml::player;

let online_players = player::online()?;
if let Some(&player_handle) = online_players.first() {
    let changed = player::grant_advancement(player_handle, "minecraft:story/root")?;
    if changed {
        // At least one criterion changed.
    }

    player::revoke_advancement(player_handle, "minecraft:story/root")?;
}
```

Both operations require `entity.read` to obtain a player handle and the separate
`player.advancement.write` permission. They run synchronously on the logical server thread and accept
only a live handle belonging to the current module. The advancement ID must be a canonical,
lowercase `namespace:path` ID of at most 256 UTF-8 bytes and must exist in the active advancement
registry. Each call affects every remaining or completed criterion, respectively, and is rejected
when that advancement defines more than 256 criteria.

Granting a complete advancement may run its configured vanilla rewards and may complete dependent
advancements. Revoking progress removes only this advancement's completed criteria; it does not
undo rewards, revoke dependent advancements, or reverse other game effects. `true` means at least
one criterion changed; `false` means the requested progress was already in that state.

Client modules do not receive these imports. Static content registration also has no live player
authority, so calls during registration fail with `PermissionDenied`.


## Criterion-level progress

For finer control, `sjml::player_advancement::progress` returns a detached, sorted snapshot of at
most 256 criteria for one registered advancement. `grant_criterion` and `revoke_criterion` change
one exact criterion after validating that key against the active advancement. The host requires
the same live server player handle and `entity.read`; criterion mutations also require
`player.advancement.write`. Completing the final criterion can trigger the normal vanilla reward
path. Revoking a criterion does not undo rewards or dependent advancements.

```rust
use sjml::player_advancement;

let progress = player_advancement::progress(player_handle, "minecraft:story/root")?;
if let Some(criterion) = progress.criteria.iter().find(|criterion| !criterion.achieved) {
    player_advancement::grant_criterion(
        player_handle,
        &progress.advancement_id,
        &criterion.id,
    )?;
}
```
