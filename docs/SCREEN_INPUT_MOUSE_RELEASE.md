# Active-screen mouse release callbacks

A client WASM module can observe and synchronously consume the release of a mouse
button delivered to the currently open Minecraft screen:

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    let _release = screen_input::subscribe_mouse_released(|event| {
        if event.button == 0 {
            // Commit a drag operation from event.mouse_x/event.mouse_y here.
            return Ok(true);
        }
        Ok(false)
    })?;
    Ok(())
}
```

The callback runs on the Minecraft client main thread from NeoForge's
`ScreenEvent.MouseButtonReleased.Pre`, before the active screen's release
handler. Returning `true` cancels that screen handler. Coordinates are scaled
GUI coordinates, button IDs are bounded to `0..=31`, and modifier bits match
the existing screen input API.

This callback has a separate opt-in subscription lane. Existing
`screen_input::subscribe` and `subscribe_key_pressed` callbacks do not receive
the new event type, so older guest decoders continue receiving only the event
tags they already understand. The host requires a client module and the
`client.input.screen_cancel` capability. It validates the grant again on each
dispatch and unsubscribe, checks the client main thread, bounds callback
registration, and removes callbacks with the owning runtime.

The release callback applies only while a screen is active. Global mouse
release input remains observational because NeoForge does not provide a
cancellable global release hook. This API does not alter mouse press, drag, or
wheel routing.

For a display-backed manual check, open a test screen with one button and
subscribe a callback that returns `true` for its left-button release. The
button's `mouseReleased` handler should not run; returning `false` should restore
vanilla handling. Also try a right-button release and confirm the callback can
leave it untouched.

