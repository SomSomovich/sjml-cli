# Player tick event

Server modules can observe a bounded `player.tick` snapshot through the shared
event subscription API:

```rust
use sjml::events::{self, EventPriority, PlayerTick};
use sjml::{event, mod_init, SjmlResult};

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe_with_interval("player.tick", EventPriority::Normal, 20)?;
    Ok(())
}

#[event(name = "player.tick")]
fn on_player_tick(tick: PlayerTick) -> SjmlResult<()> {
    let _ = (tick.player_id, tick.dimension_id, tick.tick_count);
    Ok(())
}
```

The event runs after the logical-server player tick. The snapshot contains the
canonical player UUID, current dimension ID, and non-negative server tick
count. It is observational and cannot change or cancel the tick. The plain
`events::subscribe("player.tick", priority)` form uses the same 20-tick
cadence as `server.tick`; `subscribe_with_interval` accepts `1..=1200` server
ticks. Each subscription has its own cadence. `events.subscribe` is required.

The host filters to live `ServerPlayer` instances and dispatches on the owning
server thread. Client-only and static-registration contexts cannot subscribe
to this server event. The callback follows the existing 256-subscription and
1 MiB payload bounds.
