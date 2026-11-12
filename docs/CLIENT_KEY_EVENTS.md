# Client key events

SJML exposes global keyboard observation and active-screen keyboard veto as separate client APIs with separate permissions. Both provide bounded detached values and run synchronously on the Minecraft client main thread.

## Global observation

Declare and receive `client.input.observe` to subscribe through `sjml::client_input::subscribe_global_keys`:

```rust
use sjml::{client_input, GlobalKeyObservation, SjmlResult};

fn register_keys() -> SjmlResult<()> {
    let _keys = client_input::subscribe_global_keys(|key: GlobalKeyObservation| {
        if key.action == sjml::ClientInputAction::Press && key.key_code == 32 {
            // Observe the space key.
        }
        Ok(())
    })?;
    Ok(())
}
```

The event contains a GLFW key code (`-1` means unknown), scan code, press/release/repeat action, and the six supported modifier flags. NeoForge's `InputEvent.Key` is posted after Minecraft has processed keyboard mappings and input. It is not cancellable, so this subscription cannot stop a key, change a binding, or veto vanilla processing. It can observe the notification when a screen is open as well as when gameplay has focus. Use the named keybinding API when the mod needs a user-rebindable gameplay action.

## Polling held input state

Modules with `client.input.observe` can synchronously poll whether a raw GLFW key or mouse button is held:

```rust
use sjml::{client_input, SjmlResult};

fn inspect_controls() -> SjmlResult<()> {
    if client_input::is_key_down(32)? {
        // The GLFW space-key code is currently held.
    }
    if client_input::is_mouse_button_down(0)? {
        // The primary mouse button is currently held.
    }
    Ok(())
}
```

`is_key_down` accepts GLFW key codes `0..=348`; `is_mouse_button_down` accepts button indices `0..=7`. Both require a client-capable module with `client.input.observe` declared and granted, and run on the physical client's main thread. These calls report the current held state without consuming, intercepting, or changing input. They are unavailable in dedicated-server and static-registration contexts.

## Vetoing a key delivered to an open screen

Declare and receive `client.input.screen_cancel` to use `sjml::screen_input::subscribe_key_pressed`:

```rust
use sjml::{screen_input, ScreenKeyPressedEvent, SjmlResult};

fn register_screen_keys() -> SjmlResult<()> {
    let _keys = screen_input::subscribe_key_pressed(|key: ScreenKeyPressedEvent| {
        // Return true to prevent the active screen from handling this key press.
        Ok(key.key_code == 256) // Escape
    })?;
    Ok(())
}
```

This callback runs synchronously from NeoForge's cancellable `ScreenEvent.KeyPressed.Pre`, only while a screen is active and before that screen's `keyPressed` method. `true` vetoes delivery to the screen. The detached payload contains the key code, scan code, modifier flags, and whether this is a repeat. Call `screen_input::unsubscribe_key_pressed` with the returned handle when the callback is no longer needed.

Because the result affects an event already being processed, this callback must remain synchronous and return `SjmlResult<bool>`. An `#[event_async(name = "client.screen.key_pressed")]` handler is rejected by the macro; use this typed subscription instead. The separate generic `screen_input::subscribe` API retains its existing `client.input` behavior for compatibility.

These APIs require a client-capable module and are unavailable in server-only execution. Their host imports contain no Minecraft client classes, and the event hooks are registered from the `Dist.CLIENT` bootstrap. The current project has no display-backed or headless client test harness, so unit tests validate permission gates, threading, bounded payloads, and synchronous veto aggregation; a real GLFW/NeoForge screen-key event still needs a client launch check.
