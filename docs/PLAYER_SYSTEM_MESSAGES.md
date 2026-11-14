# Player system chat messages

Server-side modules can send literal text to an online player through Minecraft's ordinary system
chat channel:

```rust
player::send_system_message(player, "The gate is open")?;
```

The host sends `ClientboundSystemChatPacket` with `overlay = false`. The message appears in chat,
not the action bar or title overlay. It is an unsigned server message; it is not a chat message
signed by the player. Text is wrapped as a literal `Component` and is never parsed as JSON, a
translation key, or a command.

For styled literal text nodes, nested children, hover text, and safe clicks, see
[rich player system messages](PLAYER_RICH_SYSTEM_MESSAGES.md).

Text must contain a non-whitespace character, contain no Unicode control characters, and fit
within 1,024 UTF-8 bytes and 256 Unicode scalar values. Valid Unicode, including emoji, is
preserved. Each module can send at most ten system messages in any rolling five-second window;
extra calls return `SERVICE_BUSY`.

## Permissions and target session

Declare both `entity.read` and `player.chat.system_message` in the module manifest:

```toml
[[modules]]
id = "main"
path = "modules/main.wasm"
side = "server"
permissions = ["entity.read", "player.chat.system_message"]
```

Then ask the server operator to grant both in `sjml-permissions.toml`:

```toml
[[grant]]
module = "example_mod/main"
capabilities = ["entity.read", "player.chat.system_message"]
```

The API accepts a module-owned `PlayerHandle`, for example one returned by
`player::online()`. The call is available only to logical-server modules, and the host checks that
the player is still online and that execution is on the owning server thread. Client modules and
static registration cannot send messages.

## Example command

The SDK example
[`player_system_message.rs`](../sjml-cli/sjml-sdk/examples/player_system_message.rs) registers
`notify.first_online`, which sends a literal message to the first online player. Declare that
command in the module manifest:

```toml
[[modules.commands]]
name = "notify.first_online"
schema_version = 1
permission_level = 2
```

The host import is `sjml_player_v1.system_message(handle_ptr, handle_len, text_ptr, text_len)` and
returns an ABI status code.
