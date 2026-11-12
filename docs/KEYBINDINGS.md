# Client keybindings

SJML modules can declare vanilla key mappings before content registration freezes, then attach bounded Rust callbacks during `mod_init`. The mappings use NeoForge's `RegisterKeyMappingsEvent`, so they appear in Minecraft's Controls screen and participate in Minecraft/NeoForge conflict handling.

```rust
use sjml::client_keybindings::{self, Context, Key, KeybindingDefinition, Modifier};
use sjml::SjmlResult;

#[mod_register]
fn register() -> SjmlResult<()> {
    client_keybindings::register(
        "open_menu",
        KeybindingDefinition::new(Key::Keyboard(77))
            .modifier(Modifier::Control)
            .context(Context::InGame),
    )?;
    Ok(())
}

#[mod_init]
fn init() -> SjmlResult<()> {
    let _subscription = client_keybindings::subscribe("open_menu", || {
        // Runs once on a client key-down. Keep work bounded and return promptly.
        Ok(())
    })?;
    Ok(())
}
```

Declare `client.input` in the module permissions. A client-only module (`side = "client"`) is the usual choice; a `side = "both"` module may also provide a client runtime and register mappings there. A server-only module cannot register mappings or callbacks. When SJML is installed on a dedicated server, the common staging code has no Minecraft client references and the mapping subscriber is loaded only for `Dist.CLIENT`.

`Key::Keyboard` accepts GLFW key-symbol codes from `-1` (unbound) through `65535`. `Key::Mouse` accepts button indices `0..=31`. `Modifier` is `None`, `Shift`, `Control`, or `Alt`. `Context` selects `InGame`, `Gui`, or `Universal`; Minecraft activates mappings according to that context. Mappings are namespaced by package and module, limited to 32 declarations per module, and paths must be 1–48 lowercase ASCII letters, digits, `_`, `.`, or `-`, starting with a letter or digit.

Callbacks receive no untrusted event payload: the host sends only a versioned callback identity after `KeyMapping.consumeClick()` reports a press. Each module may own up to 128 subscriptions. The mapping is a normal Minecraft key mapping, so users can reassign it and Minecraft handles conflicts with its built-in and other mod mappings. The existing `sjml::client_input` subscriptions remain available for raw key and mouse observations alongside named keybinding callbacks.

Key labels can be localized using `key.sjml.<package>.<module>.<path>` translation keys. The Controls category is `key.categories.sjml`.
