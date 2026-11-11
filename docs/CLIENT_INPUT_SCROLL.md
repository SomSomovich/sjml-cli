# Client mouse-wheel input

Client modules can subscribe to global mouse-wheel movement with
`sjml::client_input::subscribe_scroll`. The host reports bounded horizontal and
vertical GLFW deltas as `MouseScrollEvent { delta_x, delta_y }`; fractional
trackpad values are preserved. Each axis must be finite and in `-64.0..=64.0`,
and at least one axis must be non-zero.

The callback runs on Minecraft's client main thread outside screens. It is
observational: returning from it never cancels or changes vanilla input. A
module needs the `client.input` permission and must keep the returned handle to
unsubscribe with `sjml::client_input::unsubscribe_scroll`.

```rust,ignore
let wheel = sjml::client_input::subscribe_scroll(|event| {
    if event.delta_y > 0.0 {
        // Scroll toward the next entry.
    }
    Ok(())
})?;

sjml::client_input::unsubscribe_scroll(wheel)?;
```

The input subscription is client-only and is removed with its module runtime.
The API does not expose raw GLFW callbacks, screen cursor coordinates, or a way
to consume scroll input.
