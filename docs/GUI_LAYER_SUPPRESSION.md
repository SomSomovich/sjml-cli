# Suppressing a named GUI layer

A client module can cancel one NeoForge GUI render layer by its resource ID and
draw a replacement overlay with the existing HUD render callback. The host
reads only the ID from the guest, matches it against the layer ID from
`RenderGuiLayerEvent.Pre`, and cancels the event. No Minecraft object or
`GuiGraphics` value crosses the guest boundary, and the event hook does not
invoke guest modules.

This uses two independent permissions: `client.render.gui_layer.suppress` for
the cancellation handle and `client.render` for drawing the replacement.
Declare and grant both to a client module:

```toml
[[modules]]
id = "overlay"
side = "client"
permissions = ["client.render", "client.render.gui_layer.suppress"]
```

```toml
[[grant]]
module = "example/overlay"
capabilities = ["client.render", "client.render.gui_layer.suppress"]
```

Then cancel a named layer such as vanilla chat and subscribe to the normal HUD
pass:

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize_overlay() -> SjmlResult<()> {
    let _chat_layer = render::suppress_gui_layer("minecraft:chat")?;
    let _replacement = render::subscribe(RenderLayer::Hud, |context| {
        context.draw_rect(8.0, 8.0, 220.0, 32.0, 0xDD101820)?;
        context.draw_text("Custom chat panel", 16.0, 17.0, 0xFFFF_FFFF, true)?;
        Ok(())
    })?;
    Ok(())
}
```

The ID must be fully qualified as `namespace:path`, contain only lowercase
Minecraft resource-ID characters, and fit within 256 UTF-8 bytes. For example,
`minecraft:chat` selects the vanilla chat layer. `suppress_gui_layer` returns a
module-scoped `GuiLayerSuppressionHandle`; pass it to
`render::unsuppress_gui_layer` to restore the layer early. Shutting down or
unloading the module automatically removes its suppression handles.

Suppression is checked on the client render thread. The host keeps an exact
ID-to-registration index and performs one lookup at the layer's pre-render
event; it does not scan or call render modules for unrelated layers. A HUD
replacement uses the existing HUD post pass, so it draws after the normal GUI
layers rather than at the original layer's position. The feature changes
rendering only; it does not replace chat input or chat behavior. Layers hidden
by other Minecraft UI state, such as F1, cannot be forced visible.
The host rechecks the suppression capability at each matching pre-render
event. Revoking it stops cancellation immediately; restoring it reactivates
any suppression handle the module still owns.

The separate guest example is `sjml-sdk/examples/gui_layer_replacement.rs`.
Build it with the SDK workspace's normal `wasm32-unknown-unknown` target and
load its client module with both permissions. Visible behavior still needs a
display-backed Minecraft client run.
