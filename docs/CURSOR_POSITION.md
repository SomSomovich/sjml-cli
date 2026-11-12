# Physical-client cursor position

Client modules can call `sjml::client_input::cursor_position()` to read the
current mouse coordinates in Minecraft's scaled GUI coordinate space. The
origin is the logical viewport's upper-left corner; the returned `x` and `y`
values use the same coordinates as Minecraft screens and GUI render callbacks.
The result is a detached `CursorPosition` value, so it does not expose a
Minecraft object or retain a native window reference.

The module must request and receive `client.input`. Calls are available only
in a physical-client module runtime and must run on the client main thread. The
host samples the native cursor only while the window is live and the pointer is
inside it. When the client window is unavailable, coordinates are out of
bounds, or the host cannot produce a valid snapshot, the call returns an
error. It does not move, capture, or otherwise change the cursor.

```rust
use sjml::client_input;

fn update_pointer_target() -> sjml::SjmlResult<()> {
    let cursor = client_input::cursor_position()?;
    let hovered = cursor.x >= 20.0 && cursor.x < 180.0
        && cursor.y >= 20.0 && cursor.y < 48.0;
    if hovered {
        // Update module-owned GUI state here.
    }
    Ok(())
}
```

At the ABI boundary, `sjml_client_input_v1.cursor_position` accepts a pointer
to a 16-byte output area and returns an `i32` status. A successful call writes
two little-endian IEEE-754 doubles: X at byte offset 0, then Y at byte offset 8.
The host validates the full output span before writing and rejects non-finite,
negative, or excessively large coordinates.
