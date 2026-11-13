# Player rich system messages

`player::send_rich_system_message` sends a typed Minecraft text-component tree to a live player
through the ordinary unsigned system-chat channel:

```rust
use sjml::{player, RichText, RichTextClickAction, RichTextColor, RichTextNode};

let hover = RichText::plain("Opens the home command suggestion")?;
let link = RichTextNode::new("Set home")?
    .color(RichTextColor::Gold)?
    .bold(true)
    .click(RichTextClickAction::suggest_command("/sethome")?)
    .hover_text(hover)?;
let mut message = RichText::new();
message.push(link)?;
player::send_rich_system_message(player_handle, &message)?;
```

Nodes contain literal strings, ordered nested children, optional RGB or named chat color, and
bold, italic, underline, or strikethrough styling. Hover content uses `show_text` and accepts a
bounded rich-text tree. Supported clicks are `open_url`, `suggest_command`, and
`copy_to_clipboard`; `open_url` accepts only HTTP(S) URIs with a host and without user info.
`run_command` is not part of the API. Text is never parsed as JSON, a translation key, or a
command, and no Minecraft object crosses the guest boundary.

The host revalidates the tree and limits it to 8 levels, 64 nodes including hover nodes, 1,024
UTF-8 bytes per string, 4,096 aggregate UTF-8 string bytes, and 8,192 encoded bytes. Strings
reject control characters. The complete visible text must be nonblank. The rich and plain
system-message calls share a limit of ten sends per module per rolling five seconds.

## Permissions and execution

Declare and receive both `entity.read` and `player.chat.system_message`. Calls are available only
to server modules on the owning server thread and require a current module-owned player handle.
The host rejects stale or foreign handles. Client modules and static registration cannot send
messages.

## ABI

The host import is
`sjml_player_v1.rich_system_message(handle_ptr, handle_len, payload_ptr, payload_len) -> i32`.
The handle uses the standard 24-byte ABI encoding. The payload starts with little-endian `u32`
schema version `1`, followed by a stable length-prefixed list of top-level nodes. Each node
contains a length-prefixed UTF-8 literal, style flag byte, color tag/value, click tag and optional
string, optional hover-node list, and ordered child-node list. Color tag `0` is absent, `1` is
24-bit RGB (`u32`), and `2` is a named color (`u8`, black through white in the standard Minecraft
chat palette). Style bits are bold `1`, italic `2`, underline `4`, and strikethrough `8`. Click
tags are absent `0`, open URL `1`, suggest command `2`, and copy to clipboard `3`.

The host returns the standard ABI status code. See [the SDK example](../sjml-sdk/examples/player_rich_system_message.rs).
