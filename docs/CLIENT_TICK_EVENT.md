# Client tick event

Physical-client modules can subscribe to `client.tick` through the standard
event API:

```rust
use sjml::events::{self, ClientTick, EventPriority};
use sjml::{event, mod_init, SjmlResult};

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe_with_interval("client.tick", EventPriority::Normal, 20)?;
    Ok(())
}

#[event(name = "client.tick")]
fn on_client_tick(tick: ClientTick) -> SjmlResult<()> {
    let _tick_count = tick.tick_count;
    Ok(())
}
```

The schema-v1 payload contains a non-negative client tick counter. The callback
runs after the physical client's tick on its main thread; it cannot modify or
cancel the tick. `events::subscribe` uses a 20-tick default cadence, while
`subscribe_with_interval` accepts `1..=1200` client ticks. Each subscription
has its own cadence.

The module must request and receive `events.subscribe` and execute in the
physical-client context. Server and static-registration modules cannot
subscribe to this event. The callback uses the existing bounded subscription
and payload limits. See [gameplay events](GAMEPLAY_EVENTS.md).
