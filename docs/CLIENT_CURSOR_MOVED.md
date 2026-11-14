# Sampled client cursor movement

Client modules can subscribe through `sjml::client_input::subscribe_cursor_moved` with the
`client.input.observe` grant. Each callback receives `ClientCursorMovedEvent` with the current
scaled GUI coordinates and the delta since the previous sample.

This API intentionally samples the cursor once in `ClientTickEvent.Post`, so a callback runs at most
once per client tick and may coalesce multiple physical movements. Stationary samples do not
dispatch. Screen identity changes, window resizes,
missing cursor coordinates, and unavailable window dimensions reset the baseline without emitting
a delta. Jumps above 4,096 logical pixels on either axis are discarded and establish a new baseline.

At most one cursor listener can be active per module and the physical client admits at most 256
listeners total. Callbacks are observational and run on the client main thread. This API does not
intercept input or change vanilla handling.

For a separate callback on each native cursor event, use
[`raw physical-client cursor motion`](CLIENT_CURSOR_MOTION.md). It uses window pixels and its own
`client.input.cursor_motion` grant; this sampled API remains unchanged.

```rust,ignore
let cursor = sjml::client_input::subscribe_cursor_moved(|movement| {
    // movement.x / movement.y are the current logical GUI coordinates.
    // movement.delta_x / movement.delta_y are bounded per-tick deltas.
    Ok(())
})?;

sjml::client_input::unsubscribe_cursor_moved(cursor)?;
```
