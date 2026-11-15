# Container open and close notifications

Server modules can observe when a server player opens or closes a container
menu:

```rust
use sjml::{
    event,
    events::{self, EventPriority},
    player_container_events::{PlayerContainerClosed, PlayerContainerOpened},
    SjmlResult,
};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("player.container_open", EventPriority::Normal)?;
    events::subscribe("player.container_close", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.container_open")]
fn observe_container_open(event: PlayerContainerOpened) -> SjmlResult<()> {
    if event.menu_id == "minecraft:generic_9x1" {
        // Inspect event.player_id, event.dimension_id, event.container_id,
        // or event.slot_count here.
    }
    Ok(())
}

#[event(name = "player.container_close")]
fn observe_container_close(event: PlayerContainerClosed) -> SjmlResult<()> {
    if event.menu_id == "minecraft:generic_9x1" {
        // The close snapshot describes the same menu that was open before.
    }
    Ok(())
}
```

Both notifications run synchronously on the owning logical-server thread
through NeoForge's `PlayerContainerEvent.Open` and `PlayerContainerEvent.Close`.
NeoForge fires these events on both logical sides, but SJML dispatches only for
logical servers and only while the owning server thread is active. The events
are observational: they are not cancellable, and the handlers return `()`
rather than a veto. Changing a container from these callbacks is not supported;
use the menu APIs instead.

The detached schema-v1 payload contains the player's canonical UUID, the
dimension ID, the canonical registry ID of the menu type, the container ID
(`0..=255`), and the number of synchronized slots (`0..=512`). IDs must be
canonical lowercase resource locations; strings are bounded to 256 UTF-8 bytes
and the payload to 2 KiB. Up to 64 subscribed handlers run per notification.
Subscriptions require `events.subscribe` and the separate observe capability
`player.container.observe`; `#[event_async]` is rejected because the snapshot
is only valid while NeoForge dispatches the hook.

GameTests post a real `PlayerContainerEvent.Open` and `PlayerContainerEvent.Close`
for a vanilla chest menu and assert the detached player, dimension, menu,
container, and slot values. Unit tests cover the stable codec, bounds, UTF-8
errors, the observational NeoForge contract, and detached snapshot values.
