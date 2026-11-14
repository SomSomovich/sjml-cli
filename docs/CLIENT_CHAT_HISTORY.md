# Client chat history reads

`client_chat_history::read_recent(limit)` reads up to 64 newest entries from
Minecraft 1.21.1's physical-client `ChatComponent` display backlog. The host
copies each entry's `Component#getString()` plain text and `GuiMessage.addedTime`
GUI tick value into a detached schema-v1 snapshot. Order zero is newest. The
GUI tick is a wrapping client counter, not a date, server tick, or wall-clock
timestamp.

Declare and grant `client.chat.history.read`. Calls run only in a physical
client context on the client main thread. The API is read-only: it does not
clear, alter, send, or filter any chat. It reads the in-memory display backlog,
not the separate sent-command recall list, chat logs, or server history.

```rust
use sjml::prelude::*;

let history = client_chat_history::read_recent(20)?;
for message in history.messages {
    host::log(
        LogLevel::Debug,
        &format!("[{}] {}", message.order, message.text),
    )?;
}
```

Each entry contains plain UTF-8 text, `order`, the raw `added_tick`, and a
`truncated` flag. Text is limited to 384 UTF-8 bytes per entry, snapshots to 64
entries, and each encoded result to 32 KiB. Truncation preserves UTF-8 code
point boundaries. The payload includes no `Component`, style, click or hover
action, message signature, tag, or other Minecraft object.

The source is `ChatComponent.allMessages`, which retains up to 100 messages for
the chat display in newest-first order. It is not the set of pixels currently
visible on screen: wrapped lines, opacity, hidden-chat settings, scrolling,
and screen layout are not reported. The display may be cleared by vanilla or
the user, and only messages still retained there can be returned.

Automated checks use detached `GuiMessage` fixtures and codec tests. They run
headless and do not capture a live game window; the client accessor is tied to
the 1.21.1 `ChatComponent.allMessages` field and should be rechecked if that
vanilla display implementation changes.
