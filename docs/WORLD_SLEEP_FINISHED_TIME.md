# `world.sleep_finished_time`

`world.sleep_finished_time` reports that the logical server decided how much time to
skip after every player finished sleeping. SJML listens to NeoForge 21.1.265's
`SleepFinishedTimeEvent`, which Minecraft posts from the server level tick once the
sleep status reports enough sleeping players. The notification is observation-only:
the event is not an `ICancellableEvent` and exposes no result, so a callback cannot
change, delay, or cancel the proposed wake-up time.

The callback carries only detached values: the canonical dimension resource
location, the `new_time` the server proposes to set, and the level's current
`current_time`. No `Level`, `ServerLevel`, player, or other Minecraft object
crosses into guest code.

## Permissions and execution

Declare the subscription permission in the module manifest:

```toml
permissions = ["events.subscribe"]
```

The server operator must grant it to the module as well:

```toml
[[grant]]
module = "example_mod/main"
capabilities = ["events.subscribe"]
```

The event only runs in an integrated or dedicated server context and is delivered
on the logical server's owning thread. A client-only module does not receive it,
and a capture attempted off the owning thread is rejected. Because the handler
cannot veto, an `#[event_async]` handler is also allowed and receives the same
immutable snapshot.

Each module may hold up to 256 event subscriptions. For one sleep-finish decision,
the host invokes at most 64 subscribed callbacks. The complete payload is limited
to 1 KiB, the dimension ID to 256 UTF-8 bytes, and each time to
+/-1,000,000,000,000 ticks.

## Rust handler

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("world.sleep_finished_time", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "world.sleep_finished_time")]
fn sleep_finished(event: WorldSleepFinishedTime) -> SjmlResult<()> {
    host::log(
        LogLevel::Info,
        &format!(
            "{} sleep finished: new_time={}, current_time={}",
            event.dimension_id, event.new_time, event.current_time,
        ),
    )?;
    Ok(())
}
```

The compilable SDK example is
`sjml-cli/sjml-sdk/examples/gametest/world_sleep_finished_time.rs`.

## Schema v1

The callback export is `sjml_v1_event_world.sleep_finished_time`. Its payload
fields use the stable little-endian encoding:

1. `i32(1)` schema version.
2. Length-prefixed UTF-8 canonical dimension resource location.
3. `i64` `new_time`: the world time proposed for the wake-up.
4. `i64` `current_time`: the level day time when the event was captured.

All strings use a little-endian `u32` byte length followed by UTF-8 bytes. The
callback cannot apply either time; the server continues with its own wake-up
logic regardless of what the guest does with the notification.
