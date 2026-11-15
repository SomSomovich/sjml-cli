# Inbound player chat attempts

Server modules can veto a real inbound chat message before Minecraft broadcasts
it:

```rust
use sjml::{event, events::{self, EventPriority, PlayerChatAttempt}, SjmlResult};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("player.chat_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.chat_attempt")]
fn block_spam(event: PlayerChatAttempt) -> SjmlResult<bool> {
    Ok(event.raw_text.contains("advertisement"))
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's cancellable `ServerChatEvent`, after Minecraft has verified the
signed chat packet. Returning `Ok(true)` cancels the whole message: no client
receives it and later handlers never observe it. Returning `Ok(false)` leaves
the vanilla result unchanged. The callback cannot rewrite the message, resend
signed content, or inspect the sender's chat session.

The detached schema-v1 payload is `u32` version, a length-prefixed player UUID
string, the game-profile name (1–64 UTF-8 bytes), and the original unsigned raw
text (0–1,024 UTF-8 bytes). No Minecraft objects, signatures, or chat components
are exposed. Values outside those bounds are rejected before dispatch. Up to 64
subscribed handlers run per attempt, and all subscriptions require both
`events.subscribe` and `events.cancel`. `#[event_async]` is rejected because the
veto decision must be applied before the chat hook returns.

This is the server-side counterpart of the physical-client
`client.chat.send_attempt` veto. Client filtering, rewriting, and display
customization remain separate `client.chat.*` contracts.

GameTests post a real `ServerChatEvent` for a mock server player and verify that
the guest veto cancels only the marked message while an ordinary message keeps
its vanilla result. Unit tests cover the stable codec, bounds, the cancellable
NeoForge contract, and detached snapshot values.
