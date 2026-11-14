# Entity level lifecycle

`entity.left_level` reports that a server-side entity left one particular `ServerLevel`. The host
adapts NeoForge 21.1.265's `EntityLeaveLevelEvent`, which fires from
`LevelCallback.onTrackingEnd(Object)` whenever an entity leaves a level. The event carries the
specific level it is leaving; during a dimension transfer this is the source level. It is not
cancellable and has no result.

Leaving a level can occur during a dimension transfer, chunk unload, or another removal from that
level. The callback does not assert that an entity died or despawned, and it does not include the
entity removal reason. A later event may report that the same entity leaves another level.

## Snapshot and execution

The host filters to a `ServerLevel` and its owning server thread, then copies the entity UUID, type
resource location, the event's level dimension ID, and the entity's X/Y/Z position. All values are
detached: the event does not expose an `Entity`, `Level`, or host handle. The position must contain
finite `f64` values; invalid snapshots are skipped. The adapter uses the event's level for the
dimension ID rather than inferring it from the entity's current level.

Declare and receive the `events.subscribe` capability. The callback is observation-only and runs
synchronously on the logical-server thread. `#[event_async]` is allowed and recommended when the
handler does follow-up work: its wrapper queues the guest future and returns. That work still runs
on SJML's server main-thread lane, and the notification itself does not grant background access to
Minecraft state.

The host invokes at most 64 subscribed callbacks for one source event. Schema-v1 payloads are
limited to 588 bytes. Entity UUIDs must be canonical lowercase UUID strings; entity type and
dimension IDs must be canonical resource locations, each at most 256 UTF-8 bytes. Coordinates are
three finite IEEE-754 `f64` values.

## Rust handler

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.left_level", EventPriority::Normal)?;
    Ok(())
}

#[event_async(name = "entity.left_level")]
async fn entity_left_level(event: EntityLeftLevel) -> SjmlResult<()> {
    host::log(
        LogLevel::Info,
        &format!(
            "entity {} ({}) left {} at [{}, {}, {}]",
            event.entity_id,
            event.entity_type_id,
            event.dimension_id,
            event.x,
            event.y,
            event.z,
        ),
    )?;
    Ok(())
}
```

The SDK source example is
[`entity_left_level.rs`](../sjml-cli/sjml-sdk/examples/entity_left_level.rs).

## Payload schema v1

The callback export is `sjml_v1_event_entity.left_level`. Its little-endian fields are:

1. `i32(1)` schema version.
2. Length-prefixed canonical entity UUID string.
3. Length-prefixed entity type resource location.
4. Length-prefixed resource location of the `ServerLevel` being left.
5. Entity X, Y, and Z as finite `f64` values.

Each string is encoded as a little-endian `u32` UTF-8 byte length followed by its bytes. No live
game object crosses the guest ABI.
