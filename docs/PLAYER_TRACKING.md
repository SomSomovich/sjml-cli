# Player tracking events

The server event bus reports these observation-only notifications:

- `player.start_tracking` when NeoForge posts `PlayerEvent.StartTracking` after a `ServerPlayer`
  starts receiving updates about an entity.
- `player.stop_tracking` when NeoForge posts `PlayerEvent.StopTracking` after a `ServerPlayer`
  stops receiving updates about that entity.

Both callbacks contain only a canonical `player_id` UUID, a canonical `dimension_id`, a canonical
`target_id` UUID, and a canonical `target_type_id` resource location. The host copies those fields
directly from the event and does not expose or retain the source `Player`, target `Entity`, level,
or server. There is no cancellation result.

## Lifecycle and execution

NeoForge 21.1.265 fires both events only for the logical server. `StartTracking` runs after the
chunk map adds the entity to the player's tracking set, while `StopTracking` runs when the entity
leaves that set because it moved out of range, changed dimension, was removed, or the player
disconnected. A stop notification therefore does not imply death, despawn, or permanent removal.
SJML extracts the fields immediately on the owning logical-server thread and passes only that
detached snapshot to authorized subscribers. Client contexts, mismatched dedicated/integrated
hosts, players that are not in a `ServerLevel`, and callbacks arriving off the owning server thread
never receive a snapshot.

The regular `#[event]` callback runs synchronously on the server thread. Prefer `#[event_async]`
for follow-up work: its wrapper validates the detached payload, queues the guest future, and
returns. The future still runs on SJML's logical-server main-thread task lane, so it does not grant
background-thread access to Minecraft state. Use the `target_id` only as an identity snapshot; it
does not grant a live entity handle and must not be used to force a chunk or level load.

Only modules with an active `player.start_tracking` or `player.stop_tracking` subscription and the
`events.subscribe` grant receive callbacks. No cancellation capability is accepted. The host
invokes at most 64 callbacks for one source event. Each schema-v1 payload is at most 768 bytes: an
`i32` schema version, a length-prefixed canonical player UUID (maximum 256 bytes), a
length-prefixed canonical dimension resource location (maximum 256 bytes), a length-prefixed
canonical target UUID (maximum 256 bytes), and a length-prefixed canonical target type resource
location (maximum 256 bytes).

## Rust example

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("player.start_tracking", EventPriority::Normal)?;
    events::subscribe("player.stop_tracking", EventPriority::Normal)?;
    Ok(())
}

#[event_async(name = "player.start_tracking")]
async fn on_start_tracking(event: PlayerStartTracking) -> SjmlResult<()> {
    host::log(
        LogLevel::Info,
        &format!(
            "player {} in {} tracks {} ({})",
            event.player_id, event.dimension_id, event.target_id, event.target_type_id
        ),
    )?;
    Ok(())
}

#[event_async(name = "player.stop_tracking")]
async fn on_stop_tracking(event: PlayerStopTracking) -> SjmlResult<()> {
    host::log(
        LogLevel::Info,
        &format!(
            "player {} in {} stopped tracking {} ({})",
            event.player_id, event.dimension_id, event.target_id, event.target_type_id
        ),
    )?;
    Ok(())
}
```

The SDK source example is
[`player_tracking.rs`](../sjml-cli/sjml-sdk/examples/gametest/player_tracking.rs).

## Payload schema v1

The event exports are `sjml_v1_event_player.start_tracking` and
`sjml_v1_event_player.stop_tracking`. Both use the same little-endian payload:

1. `i32(1)` schema version.
2. Player UUID: `u32` UTF-8 byte length followed by the canonical lowercase UUID bytes.
3. Dimension ID: `u32` UTF-8 byte length followed by the canonical resource-location bytes.
4. Target UUID: `u32` UTF-8 byte length followed by the canonical lowercase UUID bytes.
5. Target type ID: `u32` UTF-8 byte length followed by the canonical resource-location bytes.

The event name identifies whether the player started or stopped tracking the entity; no live game
object crosses the guest ABI.
