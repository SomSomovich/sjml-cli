# Player command attempts

Server modules can veto a player-issued command after parsing and before
execution:

```rust
use sjml::{event, events::{self, EventPriority, PlayerCommandAttempt}, SjmlResult};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("player.command_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.command_attempt")]
fn block_marked_command(event: PlayerCommandAttempt) -> SjmlResult<bool> {
    Ok(event.command.starts_with("setblock "))
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's cancellable `CommandEvent`. It is dispatched only when the command
source's entity is a server player; console, command-block, and other
non-player sources are ignored. Returning `Ok(true)` prevents execution.
Returning `Ok(false)` leaves the parsed command unchanged.

The detached schema-v1 payload contains the player UUID, the player's
dimension ID, and the raw command text as typed without the leading slash. The
text is bounded to 1,024 UTF-8 bytes; commands whose text exceeds that limit are
not dispatched. IDs must be canonical lowercase resource locations and the
payload is capped at 2 KiB. Up to 64 subscribed handlers run per attempt, and
all subscriptions require both `events.subscribe` and `events.cancel`.
`#[event_async]` is rejected because the veto must be applied before command
execution.

GameTests post a real `CommandEvent` built from the server dispatcher's parse
results for a marked `help` command and an unmarked control, verifying that
only the marked command is canceled. Unit tests cover the stable codec, bounds,
the cancellable NeoForge contract, and detached snapshot values.
