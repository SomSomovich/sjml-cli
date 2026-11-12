# Player action-bar messages

Server-side guest modules can send a short literal-text message to an online player:

```rust
sjml::player::send_action_bar(player, "The gate is open")?;
```

The argument is sent as literal text through Minecraft's action-bar system-message route. The API
does not parse component JSON, translation keys, commands, or formatting instructions. Text must
contain 1–256 Unicode scalar values, fit within 1,024 UTF-8 bytes, contain at least one
non-whitespace character, and contain no control characters. Minecraft may apply its normal
client-side rendering behavior to the literal characters.

The module must request `entity.read` and `player.ui.actionbar`, and an administrator must grant
both capabilities. The module must run on the logical server and use a live player handle owned by
that module. The host checks the current login session and owning server thread on every call.
Client modules and static registration cannot send action-bar messages. Each module may send at
most ten messages in any rolling five-second window; excess calls return `SERVICE_BUSY`.

The player API uses the `sjml_player_v1.action_bar` import with a player handle pointer and length,
followed by a UTF-8 text pointer and length, and returns an ABI status code.
