# Active-screen mouse-wheel input

Client modules using `screen_input::subscribe` also receive mouse-wheel events
from the active screen. `ScreenInputEvent::MouseScrolled` contains the scaled
GUI cursor position and horizontal/vertical deltas. Fractional trackpad values
are preserved; both deltas must be finite and within ±64, and they cannot both
be zero.

Return `true` from the existing screen-input callback to consume the scroll
event. Return `false` to let the current screen and Minecraft handle it. Calls
run on the client main thread and require both `client.input` and the
`client.input.screen_scroll` opt-in. This separate capability keeps older
screen-input modules from receiving an event variant they do not know. The API
does not change global no-screen scrolling, which is observational through
`client_input::subscribe_scroll` and requires `client.input`.

```rust,ignore
let input = sjml::screen_input::subscribe(|event| match event {
    sjml::screen_input::ScreenInputEvent::MouseScrolled {
        mouse_x,
        mouse_y,
        delta_y,
        ..
    } if delta_y != 0.0 && mouse_x >= 0.0 && mouse_y >= 0.0 => Ok(true),
    _ => Ok(false),
})?;
```
