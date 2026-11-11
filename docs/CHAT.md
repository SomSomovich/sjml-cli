# Native client chat layout

SDK 0.41.0 adds `sjml::client_chat` for ABI 1.33.0 client modules. It controls
Minecraft's native chat rectangle, including its input field. The host styles
that rectangle with a translucent framed panel only while ChatScreen is open.
When the screen is closed, the native chat render is hidden; message data,
history, scrolling, links, and command suggestions remain native. This API does
not expose messages or a replacement screen.

Request `client.chat.modify` in `sjml.toml` and grant the module in
`config/sjml-permissions.toml`:

```toml
[[grant]]
module = "somuch-chat/main"
capabilities = ["client.chat.modify", "client.render", "storage.read", "storage.write"]
```

Calls and callbacks run synchronously on the client main thread and recheck the
effective capability. Static `mod_register`, dedicated-server instances, and
native Rust targets cannot use these imports. Draw interactive move and resize
controls through the existing `render` screen layer, which runs above the native
chat and input, with its separate `client.render` grant.

```rust
use sjml::prelude::*;

fn initialize_chat() -> SjmlResult<ChatLayoutHandle> {
    client_chat::subscribe(|event| {
        match event {
            ChatScreenEvent::Opened { viewport_width, viewport_height, layout }
            | ChatScreenEvent::ViewportResized { viewport_width, viewport_height, layout }
            | ChatScreenEvent::Closed { viewport_width, viewport_height, layout } => {
                let _ = (viewport_width, viewport_height, layout);
                // Update your grip state; Closed must cancel an active gesture.
            }
            ChatScreenEvent::MousePressed { .. }
            | ChatScreenEvent::MouseDragged { .. }
            | ChatScreenEvent::MouseReleased { .. } => {}
        }
        Ok(false)
    })
}

fn move_chat(handle: ChatLayoutHandle, x: f32, y: f32) -> SjmlResult<ChatLayout> {
    let current = client_chat::layout(handle)?;
    client_chat::set_layout(handle, ChatLayout::new(x, y, current.width(), current.height())?)
}
```

There is one exclusive controller across client runtimes. A competing
subscription returns `ErrorCode::RegistrationLimitReached`. Keep the returned
opaque `ChatLayoutHandle` for reads, updates, and `client_chat::unsubscribe`.
Unsubscribing revokes its callback, expires every copied handle, and restores
vanilla layout control. Runtime close also expires handles and callback routes.
Closing ChatScreen only ends the current gesture; the selected session layout
and controller remain active for the HUD and subsequent ChatScreen openings.

`ChatLayout` is `Copy`/`Clone` with a checked constructor and `x()`, `y()`,
`width()`, `height()` accessors. Coordinates use logical scaled-GUI units with
a top-left origin. All fields must be finite; width and height must be positive.
Negative requested positions are permitted. The host clamps position and size
so the full pane stays visible, with an effective minimum of 120 by 56 GUI units.
For a smaller viewport each minimum shrinks to that viewport dimension. Always
use the rectangle returned by `set_layout`; window resizing reclamps it. The
host does not overwrite saved vanilla chat options. Vanilla suggestions are
anchored to the native input and clamped to the viewport; the popup may extend
outside the chat rectangle.

The lifecycle variants `Opened`, `Closed`, and `ViewportResized` carry viewport
width, viewport height, and the current rectangle. Pointer variants are
`MousePressed { mouse_x, mouse_y, button }`,
`MouseDragged { mouse_x, mouse_y, button, delta_x, delta_y }`, and
`MouseReleased { mouse_x, mouse_y, button }`. Mouse coordinates and deltas are
finite and bounded to ±1,048,576; button codes range from 0 through 31. Return
`true` only for a left-button (`0`) event handled by a move/resize grip. The SDK
forces all other buttons and lifecycle results to `false`. Keep native text
selection, links, scrolling, and input behavior available outside the grips.
Reentrant callback dispatch returns `ServiceBusy` instead of borrowing an
already active closure. Callback errors propagate through the stable ABI error
response and release the dispatch guard.

## Local profile persistence

Request and grant `storage.read` and `storage.write` separately. Use the existing
per-mod `StorageScope::ClientProfile`, which is local installation data and
independent of the current world or remote server. Store normalized fractions
in a versioned typed record, then scale them to the latest viewport on open:

```rust
use sjml::{client_chat::ChatLayout, storage::{self, StorageScope}, StableDecode, StableEncode};

#[derive(StableEncode, StableDecode)]
#[sjml(version = 1, id = "example.chat-layout")]
struct SavedLayout {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

fn restore(viewport_width: f32, viewport_height: f32) -> Result<Option<ChatLayout>, String> {
    let saved = storage::read::<SavedLayout>(&StorageScope::ClientProfile, "chat-layout")
        .map_err(|error| error.to_string())?;
    let Some(saved) = saved else { return Ok(None); };
    for value in [saved.x, saved.y, saved.width, saved.height] {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err("stored chat fractions are invalid".into());
        }
    }
    ChatLayout::new(saved.x * viewport_width, saved.y * viewport_height,
        saved.width * viewport_width, saved.height * viewport_height)
        .map(Some).map_err(|error| error.to_string())
}

fn persist(layout: ChatLayout, viewport_width: f32, viewport_height: f32)
    -> Result<(), String>
{
    if !viewport_width.is_finite() || !viewport_height.is_finite()
        || viewport_width <= 0.0 || viewport_height <= 0.0 {
        return Err("viewport must be finite and positive".into());
    }
    // Pass the current authoritative host-clamped layout.
    storage::write(&StorageScope::ClientProfile, "chat-layout", &SavedLayout {
        x: layout.x() / viewport_width,
        y: layout.y() / viewport_height,
        width: layout.width() / viewport_width,
        height: layout.height() / viewport_height,
    }).map_err(|error| error.to_string())
}
```

On `Opened`, `Ok(None)` keeps the vanilla-derived rectangle. For `Ok(Some(..))`,
apply the restored request with `set_layout` and retain its clamped return value.
On a restore error, log the failure, fall back to vanilla geometry, and disable
persistence for that session so a read failure or corrupt record is not
silently overwritten. Save only after a completed gesture or `Closed`, never on
every drag event. A write failure leaves the current session geometry active;
log it without replacing the current rectangle.

## Verification boundary

Codec, registry, bounds, and host tests establish the wire and lifecycle
contracts. Successful Rust/Java tests and a packaged guest do not prove native
Minecraft visuals. A live client must still verify chat wrapping, scroll
position, links and hover hit testing, drag capture, input, suggestions, GUI
scaling, window resizing, relog, and restart persistence. No live-client
verification is claimed here.
