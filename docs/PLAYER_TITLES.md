# Player title and subtitle messages

Logical-server modules can send literal Minecraft title and subtitle packets to a live player with
`player::show_title`:

```rust
use sjml::{player, PlayerTitleTimings};

player::show_title(
    player_handle,
    "The gate is open",
    Some("Welcome back"),
    PlayerTitleTimings::new(10, 70, 20)?,
)?;
```

The module must declare and receive `entity.read` and `player.ui.title`. The call validates the live
player session and runs on the owning server thread. Client modules and static-registration
callbacks cannot send titles. Every module is limited to ten title sends per five-second window.

Title and subtitle are literal plain text, not JSON, translation keys, or commands. The title must
not be blank; the subtitle is optional and can be an empty string. Each value is limited to 1,024
UTF-8 bytes and 256 Unicode scalar values, and control characters are rejected. Fade-in, stay, and
fade-out durations are expressed in ticks and each must be between zero and 6,000. The host sends
the standard animation, title, and subtitle packets; it does not modify signed chat or scoreboard
state.

The guest ABI import is `sjml_player_v1.title` with the player handle, two UTF-8 text spans, and
three `i32` timing values. The host returns an error rather than truncating either text value or
accepting out-of-range timing data.
