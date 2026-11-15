# `world.difficulty_change`

`world.difficulty_change` reports that the logical server's difficulty changed.
SJML listens to NeoForge 21.1.265's `DifficultyChangeEvent`, which is posted
after the server applies the new difficulty and updates mob-spawning flags and
online players. The event is notification-only: it is not an
`ICancellableEvent` and exposes no result or setter, so a callback cannot change
or cancel the new difficulty.

The NeoForge hook carries no level. SJML dispatches this notification only when
a current server exists and the calling thread is that server's thread, so the
guest receives only the detached new and previous difficulty names. Difficulty
is shared server world data, not a per-dimension setting.

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

The event only runs in an integrated or dedicated server context. It is
delivered on the owning server thread; a client-only module does not receive
it. Because the handler cannot veto, a `#[event_async]` handler is also
allowed and receives the same immutable snapshot.

Each module may hold up to 256 event subscriptions. For one difficulty change,
the host invokes at most 64 subscribed callbacks. The complete payload is
limited to 256 bytes, and each difficulty string is limited to 64 UTF-8 bytes.

## Rust handler

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("world.difficulty_change", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "world.difficulty_change")]
fn difficulty_changed(event: WorldDifficultyChange) -> SjmlResult<()> {
    host::log(
        LogLevel::Info,
        &format!(
            "difficulty changed from {} to {}",
            event.old_difficulty, event.new_difficulty,
        ),
    )?;
    Ok(())
}
```

The compilable SDK example is
[`world_difficulty_change.rs`](../sjml-sdk/examples/gametest/world_difficulty_change.rs).

## Schema v1

The callback export is `sjml_v1_event_world.difficulty_change`. Its payload
fields use the stable little-endian encoding:

1. `u32(1)` schema version.
2. Length-prefixed UTF-8 `new_difficulty`: the lowercase difficulty name after
   the change, one of `peaceful`, `easy`, `normal`, or `hard`.
3. Length-prefixed UTF-8 `old_difficulty`: the lowercase difficulty name before
   the change.

All strings use a little-endian `u32` byte length followed by UTF-8 bytes.
