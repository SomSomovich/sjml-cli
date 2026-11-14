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
cancellable global release hook. This API does not alter mouse press or wheel
routing. Active-screen drag input has its own subscription lane, described
below.

For a display-backed manual check, open a test screen with one button and
subscribe a callback that returns `true` for its left-button release. The
button's `mouseReleased` handler should not run; returning `false` should restore
vanilla handling. Also try a right-button release and confirm the callback can
leave it untouched.

## Active-screen mouse dragging

Use `screen_input::subscribe_mouse_dragged` to receive detached pointer
coordinates, movement delta, and held button while the cursor is dragging over
the active screen:

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    let _drag = screen_input::subscribe_mouse_dragged(|event| {
        if event.button == 0 {
            // Update a guest-owned selection with event.mouse_x/event.mouse_y.
            return Ok(true);
        }
        Ok(false)
    })?;
    Ok(())
}
```

The callback runs synchronously on the client main thread from
`ScreenEvent.MouseDragged.Pre`. Returning `true` consumes the active screen's
drag handler. Coordinates use scaled GUI units and are finite and bounded;
each delta is capped at 4,096 units and button IDs at `0..=31`. The subscription
requires `client.input.screen_cancel`. It is separate from the general screen
input lane, so older guest decoders never receive the new event tag. Global
mouse movement remains observational.

