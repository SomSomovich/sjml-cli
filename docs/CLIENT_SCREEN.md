# Interactive client screens

A physical-client module can open a bounded vanilla screen with literal text and up to four
buttons. Declare `client.ui.screen` in the module manifest. Button actions are sent directly to
the owning module's `client.screen.action` callback.

The module must export `sjml_v1_event_client.screen.action` before calling `open`; the host checks
this before opening the screen and returns `INVALID_ARGUMENT` when the export is missing.

```rust
use sjml::client_screen::{self, ClientScreenButton, ClientScreenDefinition};
use sjml::events::ClientScreenAction;
use sjml::prelude::*;

#[client_command(name = "screen.open")]
fn open_screen(topic: String) -> SjmlResult<String> {
    client_screen::open(&ClientScreenDefinition {
        id: "welcome".to_owned(),
        title: "SJML client screen".to_owned(),
        body: vec![
            format!("Topic: {topic}"),
            "Choose a button to continue.".to_owned(),
        ],
        buttons: vec![
            ClientScreenButton {
                id: "continue".to_owned(),
                label: "Continue".to_owned(),
            },
            ClientScreenButton {
                id: "cancel".to_owned(),
                label: "Cancel".to_owned(),
            },
        ],
    })?;
    Ok("Opened the local screen".to_owned())
}

#[event(name = "client.screen.action")]
fn handle_screen_action(action: ClientScreenAction) -> SjmlResult<()> {
    if action.screen_id != "welcome" {
        return Ok(());
    }

    let message = match action.button_id.as_str() {
        "continue" => "You chose Continue.",
        "cancel" => "You chose Cancel.",
        _ => return Ok(()),
    };

    // The clicked screen has already closed. Open a separate result screen for this response.
    client_screen::open(&ClientScreenDefinition {
        id: "welcome.result".to_owned(),
        title: "Button received".to_owned(),
        body: vec![message.to_owned()],
        buttons: vec![ClientScreenButton {
            id: "done".to_owned(),
            label: "Close".to_owned(),
        }],
    })
}
```

The local `screen.open` command accepts a string argument and is declared in the module's
`[[modules.client_commands]]` table. Run it from Minecraft with
`/sjmlclient <package-id> <module-id> screen.open <topic>`; for example,
`/sjmlclient client_screen_demo screen screen.open welcome`.

```toml
[[modules]]
id = "screen"
path = "modules/screen.wasm"
side = "client"
permissions = ["client.ui.screen", "client.command"]

[[modules.client_commands]]
name = "screen.open"
schema_version = 1
permission_level = 0

[[modules.client_commands.arguments]]
name = "topic"
type = "greedy_string"
```

The module must request `client.ui.screen` and `client.command` in its manifest. Each capability
must also be granted by the local Minecraft instance in
`config/sjml-permissions.toml`:

```toml
[[grant]]
module = "client_screen_demo/screen"
capabilities = ["client.ui.screen", "client.command"]
```

The screen API is client-only. It does not contact a server or modify server state. Its title,
body lines, and button labels are literal text. A button click closes the screen and sends the
owning module a `ClientScreenAction` containing the screen's `screen_id` and the clicked
`button_id`. This owner-scoped callback is dispatched directly and does not use
`events::subscribe`. The callback can then update module state, open another screen, or run
another local client action. Closing a screen with Escape does not represent a button action.

## Text-entry forms

Use `client_screen::open_form` when a screen needs bounded text input. A form keeps the same
button-only screen payload and adds one to four labelled text fields:

```rust
use sjml::client_screen::{
    self, ClientScreenButton, ClientScreenDefinition, ClientScreenFormDefinition,
    ClientScreenTextField,
};
use sjml::events::ClientScreenFormSubmit;
use sjml::{event, SjmlResult};

fn show_profile_form() -> SjmlResult<()> {
    client_screen::open_form(&ClientScreenFormDefinition {
        screen: ClientScreenDefinition {
            id: "profile.form".to_owned(),
            title: "Profile".to_owned(),
            body: vec!["Set a display name.".to_owned()],
            buttons: vec![ClientScreenButton {
                id: "save".to_owned(),
                label: "Save".to_owned(),
            }],
        },
        fields: vec![ClientScreenTextField {
            id: "name".to_owned(),
            label: "Display name".to_owned(),
            initial_value: String::new(),
            max_utf8_bytes: 32,
        }],
    })?;
    Ok(())
}

#[event(name = "client.screen.form_submit")]
fn handle_form(submit: ClientScreenFormSubmit) -> SjmlResult<()> {
    let display_name = submit.fields.iter()
        .find(|field| field.id == "name")
        .map(|field| field.value.as_str())
        .unwrap_or_default();
    // Inspect submit.button_id to distinguish Save from Cancel.
    let _ = display_name;
    Ok(())
}
```

Each field has a stable ID, literal label and initial value, and a byte limit from 1 to 128.
The host filters typed text to that UTF-8 byte limit and rejects control characters. Field IDs
must be unique and cannot match a button ID. The total request stays within the same 4,096-byte
screen payload bound.

When any form button is clicked, including a button labelled `Cancel`, the screen closes and the
owner receives `ClientScreenFormSubmit` through the direct
`client.screen.form_submit` callback. The callback includes the clicked button ID and every
field's current `ClientScreenFormValue { id, value }`. It is separate from
`client.screen.action`, so button-only screens keep their existing callback payload. Escape or
other external closure sends no callback. The form callback must be exported before `open_form`
is called; otherwise the host returns `INVALID_ARGUMENT`. Neither callback requires
`events::subscribe`.

## Body scrolling

If the body text is taller than its visible area, the mouse wheel scrolls only that body viewport.
The scroll offset is clamped to the available text range. Scrolling is local screen behavior: it
does not invoke module callbacks or send network traffic. Wheel input is not consumed over text
fields, buttons, or outside the body viewport, so those controls keep their normal input behavior.
Visible layout and wheel handling still need verification in a display-backed Minecraft client.

## Bounds

Each screen has a 1–64 byte ID, a title of at most 128 UTF-8 bytes, up to six body lines of at most
160 UTF-8 bytes each, and one to four buttons. Button IDs are also 1–64 bytes; labels are at most
96 UTF-8 bytes. IDs use lowercase ASCII letters, digits, and `_.:/-`. The complete encoded screen
must fit in 4,096 bytes. Text cannot contain control characters; an empty body line is allowed.

This is a small vanilla-screen API rather than general screen construction: it does not expose
Minecraft objects, arbitrary widget trees, custom render callbacks, or focus management. Use
[`RENDERING.md`](RENDERING.md) for HUD and active-screen overlays, and
[`SCREEN_INPUT_SCROLL.md`](SCREEN_INPUT_SCROLL.md) for input delivered to the currently active
screen.
