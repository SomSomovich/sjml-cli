# Outgoing client chat veto

The physical-client `client.chat.send_attempt` event lets a module reject a
message before Minecraft sends it. It receives the original text the player
typed from NeoForge's cancellable `ClientChatEvent`. The guest cannot replace
that text, so allowed messages continue through Minecraft's ordinary
signed-chat path. The server does not need this module installed.

## Subscribe and veto

```rust
use sjml::events::{self, ClientChatSendAttempt, EventPriority};
use sjml::{event, mod_init, SjmlResult};

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("client.chat.send_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "client.chat.send_attempt")]
fn block_marked_messages(event: ClientChatSendAttempt) -> SjmlResult<bool> {
    Ok(event.message.starts_with("!blocked "))
}
```

The module must request these capabilities, and the local administrator must
grant them to the client instance:

```toml
[[modules]]
id = "main"
side = "client"
permissions = [
    "events.subscribe",
    "events.cancel",
    "client.chat.send.observe",
]
```

```toml
[[grant]]
module = "example/main"
capabilities = [
    "events.subscribe",
    "events.cancel",
    "client.chat.send.observe",
]
```

`client.chat.send.observe` is required because the callback sees user-typed
text. `events.cancel` is required because it can veto sending. The callback is
synchronous on the client main thread and receives at most 32 KiB of valid
control-free UTF-8. Up to 64 handlers run for one attempt. A `true` result
cancels the send; a callback error disables that module and preserves vanilla
behavior. A nested chat send from a guest callback is canceled to prevent
recursive callbacks and veto bypasses.

The hook does not call `ClientChatEvent.setMessage`. It only cancels the
original event, preserving Minecraft's normal chat signing and transport for
messages that are allowed. This is not an outgoing-message rewrite API.
