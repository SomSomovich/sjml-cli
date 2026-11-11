# Client chat message callbacks

`sjml::chat_messages::subscribe` receives a detached text snapshot immediately
before the physical client's vanilla chat component inserts it into local
scrollback. The callback runs synchronously on the client main thread. It
receives one of two source labels:

- `PlayerChat` for a message carrying Minecraft's player-message signature.
- `System` for unsigned text and system notices.

Minecraft has already applied its secure-chat policy, blocked-player checks,
and server-supplied filtering before this hook. The guest sees only plain text,
never a signature, `Component`, style, hover/click event, or server object. Text
is capped at 4096 UTF-8 bytes and truncated on a code-point boundary; the
`was_truncated()` flag reports that case.

The required capability is `client.chat.observe`. Returning
`ChatMessageAction::Hide` is honored only if `client.chat.filter` is also
declared in the module manifest and granted by the host. Without the second
capability, a callback can observe a message but cannot suppress it. Filtering
affects only that physical client's vanilla chat display and local scrollback;
it does not change or resend the signed message, alter the server, bypass
Minecraft's chat checks, or modify the reporting chat log. It also does not
suppress narration, overlays, or messages sent through other UI surfaces.

```rust,ignore
use sjml::chat_messages::{self, ChatMessageAction, ChatMessageSource};

let _subscription = chat_messages::subscribe(|message| {
    if message.source() == ChatMessageSource::PlayerChat
        && message.text().contains("local phrase")
    {
        Ok(ChatMessageAction::Hide)
    } else {
        Ok(ChatMessageAction::Show)
    }
})?;
```

Subscriptions are owned by the module runtime and are removed on explicit
unsubscribe or runtime shutdown. Each runtime has one subscription; the host
supports up to 128 active runtimes. A guest callback failure removes only that
runtime's subscription and leaves other modules' handlers active. Callbacks
must stay short because they run synchronously while the client adds a chat
message. This API observes messages at the display insertion point; it does
not expose outgoing chat or permit editing/reformatting message components.

The host import module is `sjml_client_chat_messages_v1`.

- `subscribe(i64, i32) -> i32` registers callback export
  `sjml_v1_client_chat_message_dispatch`.
- `unsubscribe(i32, i32) -> i32` releases the opaque kind-14 handle.

The version-1 callback payload contains the callback ID, source tag,
truncation flag, and bounded UTF-8 text; its result is one stable bool byte.
