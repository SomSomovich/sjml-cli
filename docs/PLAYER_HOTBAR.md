# Player hotbar selection

Logical-server modules can read and change which of the nine hotbar slots an online player has
selected:

```rust
use sjml::player;

let handle = player::online()?.into_iter().next().ok_or_else(|| {
    sjml::SjmlError::new(sjml::ErrorCode::InvalidArgument, "no online player")
})?;

let current = player::selected_hotbar_slot(handle)?;
player::set_selected_hotbar_slot(handle, (current + 1) % 9)?;
```

Reading requires `entity.read`. Changing selection requires both `entity.read` and
`inventory.write`. Indices are zero-based and must be in `0..=8`. The host resolves the current
online session and checks the logical-server thread on every call. Client modules, static
registration, and stale player handles cannot access the live selection.

Changing the selection does not move or replace any item stack. The host updates the server
inventory's selected index and sends Minecraft's carried-slot packet to synchronize the choice
with that player's client.
