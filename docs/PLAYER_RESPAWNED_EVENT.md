# `player.respawned` event

Server modules can subscribe to `player.respawned` to observe a player after
NeoForge completes respawn placement. The host sends a detached snapshot on the
owning logical-server thread for both dedicated and integrated servers. The
event is observational and cannot cancel or alter the respawn.

```rust
use sjml::events::{self, EventPriority, PlayerRespawned};
use sjml::{event, mod_init, SjmlResult};

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("player.respawned", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.respawned")]
fn on_player_respawned(event: PlayerRespawned) -> SjmlResult<()> {
    sjml::host::log(&format!("{} respawned in {}", event.display_name, event.dimension_id));
    Ok(())
}
```

The schema-v1 payload contains the player's canonical UUID, profile name, new
dimension ID, finite respawn coordinates, and `end_conquered`, which is true
when the respawn followed completion of the End credits. Names and dimension
IDs use the standard 64 KiB ABI string bound; the full event remains subject to
the 1 MiB callback payload limit.

The event requires only the module's existing `events.subscribe` grant. It is
not delivered in client execution contexts and does not expose a live
`ServerPlayer` or a mutable Minecraft object.
