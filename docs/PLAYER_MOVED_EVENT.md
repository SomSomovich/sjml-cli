# `player.moved` event

`player.moved` is a non-cancellable, server-authoritative notification for
meaningful player position, look, or dimension changes. The host samples a
player after the logical-server entity tick and sends an owned schema-v1
`PlayerMoved` value. It does not expose a Minecraft object or retain references
to the player.

Subscribe with the module's `events.subscribe` permission and grant:

```toml
permissions = ["events.subscribe"]
```

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    let _movement = events::subscribe("player.moved", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.moved")]
fn on_player_moved(event: PlayerMoved) -> SjmlResult<()> {
    let _snapshot = (
        event.player_id,
        event.display_name,
        event.dimension_id,
        event.x,
        event.y,
        event.z,
        event.yaw,
        event.pitch,
    );
    Ok(())
}
```

The schema-v1 record fields are `player_id` (canonical UUID string),
`display_name`, `dimension_id`, coordinates `x`, `y`, `z` as `f64`, and
`yaw`, `pitch` in degrees as `f32`. Numeric fields are finite and strings and
the complete payload obey the SDK's 64 KiB and 1 MiB stable-ABI limits.

The host seeds a baseline when a player joins. A later sample is emitted when
the player changes dimension, moves at least 1 cm from the last emitted
position, or changes yaw or pitch by at least 0.1 degrees. Sub-threshold
position changes accumulate against that baseline. Yaw comparisons wrap at
360 degrees. Each delivered event becomes the new baseline. Login, logout,
world unload, and runtime shutdown clear the relevant state.

Callbacks run synchronously on the owning logical-server thread. Dedicated
servers and integrated servers can deliver this event; client-only runtimes
cannot subscribe to it. Grant `events.subscribe` only to modules that need the
notification, since a moving player can generate callbacks on consecutive
ticks. The event reports state after movement; it does not cancel or alter
movement.
