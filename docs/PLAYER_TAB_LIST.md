# Player-list header and footer

Logical-server modules can set a literal header and footer for one online
player with `player::set_tab_list`:

```rust
use sjml::player;

player::set_tab_list(player_handle, "Server status", "Welcome to the world")?;
player::set_tab_list(player_handle, "Updated status", "")?;
player::clear_tab_list(player_handle)?;
```

The module must request and receive `entity.read` and `player.ui.tab_list`.
Each header/footer can be empty, but a `set_tab_list` call must provide at
least one nonblank field. Text is literal plain text, not JSON, a translation
key, or a command; it is limited to 1,024 UTF-8 bytes and 256 Unicode scalar
values per field, with control characters rejected. It runs on the owning
server thread, verifies the current live player session, and is limited to ten
updates per module per five-second window.

Each module owns one contribution per player. Active SJML modules' fields are
sorted by their text and joined with newlines, so update order does not change
the result. Clearing or closing one module removes only its contribution and
recomposes the remaining text. The host caps each player at 32 contributors
and each composed field at 32 KiB. Contributions are in-memory for a live
server session and are removed on logout or server shutdown.

The host sends Minecraft's normal tab-list packet with literal text
components. This API composes SJML modules; it does not provide coordination
with unrelated mods that independently send their own tab-list packet.
