# Client chat message callbacks

`sjml::chat_messages::subscribe` observes a detached snapshot of an incoming
message on the physical client main thread. It runs from NeoForge's
`ClientChatReceivedEvent` immediately before the local message is displayed.
Minecraft's secure-chat verification, blocked-player checks, and server chat
filtering have already run. Overlay/action-bar messages are not part of this
callback.

The guest receives only the message's bounded plain text, source, and
truncation flag. It never receives a signature, original `Component`, style,
hover/click event, or server object. Text is capped at 4096 UTF-8 bytes and
truncated at a code-point boundary.

The module must declare and receive `client.chat.observe` to subscribe.
Callbacks may request three local display actions:

- `ChatMessageAction::Show` keeps the current message.
- `ChatMessageAction::Hide` hides the message only for this client and
  requires the separately declared and granted `client.chat.filter`.
- `ChatMessageAction::Replace` substitutes the entire rendered message only
  for this client and requires the separately declared and granted
  `client.chat.rewrite`.

A replacement is built from at most 64 literal text spans, with a combined
4096-byte text limit. Each span can set an RGB color, bold, italic, underline,
strikethrough, obfuscation, and bounded hover text. Hover and click values are
capped at 512 bytes each and 8192 bytes combined. Click actions are limited to
HTTP(S) URL opening, suggesting text in the chat input, and copying text to the
clipboard. Minecraft keeps its normal confirmation for opening a URL.
Command execution and file opening are not exposed.

Subscriptions run in registration order. Each callback sees the latest
replacement as plain text; the last authorized replacement supplies the final
rendered spans, while any authorized Hide suppresses display. A replacement
does not alter or resend the signed server message or affect server state.

```rust,ignore
use sjml::chat_messages::{self, ChatClickAction, ChatMessage, ChatMessageAction, ChatSpan, ChatText};

let _subscription = chat_messages::subscribe(|message: ChatMessage| {
    if message.text().contains("website") {
        let link = ChatSpan::new("open site")?
            .color(0x55AAFF)?
            .bold(true)
            .hover_text("Open the project website")?
            .click(ChatClickAction::open_url("https://example.com")?);
        let mut replacement = ChatText::plain("Visit ")?;
        replacement.push(link)?;
        Ok(ChatMessageAction::Replace(replacement))
    } else {
        Ok(ChatMessageAction::Show)
    }
})?;
```

For the example, declare both `client.chat.observe` and
`client.chat.rewrite` in the client module. Use `client.chat.filter` instead
when the module should only hide matching messages. Capabilities are checked
again when each callback returns, so revoking a grant immediately disables
that action.

Callbacks must stay short because they run synchronously while the client is
processing the display event. A malformed result or guest callback failure
removes only that runtime's subscription. Each runtime may have one
subscription; the host supports up to 128 active runtimes. The callback request
uses schema 1; response values 0 and 1 preserve the old Show/Hide encoding, and
response value 2 carries a bounded styled replacement.

The host import module is `sjml_client_chat_messages_v1`.

- `subscribe(i64, i32) -> i32` registers
  `sjml_v1_client_chat_message_dispatch`.
- `unsubscribe(i32, i32) -> i32` releases the opaque kind-14 handle.

## Sending ordinary chat

`sjml::chat_messages::send("hello")` sends a normal player chat message and
requires the separate `client.chat.send` capability, a physical client, the
client main thread, and an active local player connection. The host calls
Minecraft's `LocalPlayer.connection.sendChat` path, which runs NeoForge's
client-send hook, session signing, and acknowledgement handling. The guest
cannot send packets directly or invoke command dispatch through this API.

Text must contain 1–256 UTF-16 code units and at most 1024 UTF-8 bytes. Blank
text, control characters, line breaks, and text whose first non-whitespace
character is `/` are rejected. Each module runtime may send once per second;
faster calls return `ServiceBusy`. Sending is not routed through the incoming
display callback above.

```rust,ignore
use sjml::chat_messages;

chat_messages::send("Hello from my mod")?;
```

The host import is `sjml_client_chat_send_v1.send(i32, i32) -> i32`; the two
arguments point to UTF-8 bytes in guest memory. It is independent of the
message callback subscription import and checks `client.chat.send` on every
call.

Modules that need to read and veto text the user types can separately subscribe
to `client.chat.send_attempt`. That event never changes outgoing text and uses
the local `client.chat.send.observe` grant plus `events.cancel`; see
[Outgoing client chat veto](CLIENT_CHAT_SEND.md).
