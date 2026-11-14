# Raw physical-client cursor motion

Client modules can subscribe with `sjml::client_input::subscribe_cursor_motion` after declaring and
receiving the separate `client.input.cursor_motion` capability. The callback receives a detached
`ClientCursorMotionEvent`: current window position and movement since the preceding native GLFW
cursor callback, in physical window pixels. Coordinates are not scaled to Minecraft GUI pixels.

```rust,ignore
let _motion = sjml::client_input::subscribe_cursor_motion(|movement| {
    // Use the per-event window-pixel position and delta for custom drag tracking.
    let _position_and_delta = (movement.x, movement.y, movement.delta_x, movement.delta_y);
    Ok(())
})?;
```

The host injects into Minecraft 1.21.1 `MouseHandler.onMove(long,double,double)` and routes each
valid event synchronously on the client main thread after Minecraft updates its own mouse state.
The callback cannot consume input or alter vanilla handling. Each module can hold one listener; the
physical client admits at most 64 listeners and dispatches at most 16 raw events per listener per
client tick. Further events for a listener in that tick are dropped. The event, position, and each
delta are bounded; a first callback after a window change only establishes a baseline, and an
oversized jump resets it.

Raw callbacks execute in the mouse event path, so keep handlers short and do not wait for work on
another thread. The existing `subscribe_cursor_moved` API is unchanged: it samples once per tick,
uses scaled GUI coordinates, and requires `client.input.observe`. See
[`CLIENT_CURSOR_MOVED.md`](CLIENT_CURSOR_MOVED.md) for its distinct behavior.

## Reproducible client check

Build `client_cursor_motion` as a client module with `client.input.cursor_motion` both declared in
the module permissions and granted by the host. Open any screen, move the mouse in short and long
strokes, then verify the callback sees physical-pixel positions and successive deltas. Move the
window between displays or resize it; each valid position callback continues with its actual
native coordinates. Compare with `client_cursor_moved` to see that the sampled API is still limited
to one GUI-space callback per tick. This is a display-backed manual check; the codec, tracker,
host authorization, dispatch limit, and callback routing are also covered by automated tests.
