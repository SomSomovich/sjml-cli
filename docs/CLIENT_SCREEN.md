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

## Scrollable selection lists

Use `client_screen::open_list_form` to add one bounded selection list to a screen. A
`ClientScreenListFormDefinition` contains the base screen, zero to four text fields, and one
`ClientScreenSelectionList` with a stable ID, label, and one to 32 options. Option IDs must be
unique; field and list IDs are distinct from each other and from button IDs. List and option IDs
use the usual 1–64 byte lowercase ASCII syntax; list and option labels are 1–96 UTF-8 bytes. An
initial selection may be omitted, or must name one of the options.

```rust
use sjml::client_screen::{
    self, ClientScreenButton, ClientScreenDefinition, ClientScreenListFormDefinition,
    ClientScreenSelectionList, ClientScreenSelectionOption,
};
use sjml::events::ClientScreenFormSubmit;
use sjml::{event, SjmlResult};

fn show_destination_list() -> SjmlResult<()> {
    client_screen::open_list_form(&ClientScreenListFormDefinition {
        screen: ClientScreenDefinition {
            id: "profile.destination".to_owned(),
            title: "Choose a destination".to_owned(),
            body: vec!["Pick one option, then submit.".to_owned()],
            buttons: vec![ClientScreenButton {
                id: "submit".to_owned(),
                label: "Continue".to_owned(),
            }],
        },
        fields: vec![],
        list: ClientScreenSelectionList {
            id: "destination".to_owned(),
            label: "Destination".to_owned(),
            options: vec![
                ClientScreenSelectionOption {
                    id: "overworld".to_owned(),
                    label: "Overworld".to_owned(),
                },
                ClientScreenSelectionOption {
                    id: "nether".to_owned(),
                    label: "Nether".to_owned(),
                },
                ClientScreenSelectionOption {
                    id: "end".to_owned(),
                    label: "The End".to_owned(),
                },
            ],
            initial_selected_id: Some("overworld".to_owned()),
        },
    })?;
    Ok(())
}

#[event(name = "client.screen.form_submit")]
fn handle_destination(submit: ClientScreenFormSubmit) -> SjmlResult<()> {
    let selected = submit
        .fields
        .iter()
        .find(|value| value.id == "destination")
        .map(|value| value.value.as_str())
        .unwrap_or_default();
    let _ = selected;
    Ok(())
}
```

The mouse wheel scrolls the list locally when its visible option window is shorter than the
option set. Changing the selection or scrolling does not invoke guest callbacks or send network
traffic.
When a button is clicked, the owner receives the selected option as one additional
`ClientScreenFormValue { id, value }` in `ClientScreenFormSubmit.fields`, after the text-field
values. Its `id` is the list ID and its `value` is the selected option ID; no selection is sent as
an empty string. This happens only on button submission, including a `Cancel` button. Escape
closes without submitting. The list uses the existing owner-only `client.screen.form_submit`
callback and needs no separate capability. Its appearance and wheel interaction still need
display-backed verification. The `client.screen.form_submit` export must be present before
opening the list form; otherwise the host returns `INVALID_ARGUMENT`.
Schema v3 is additive: schema-v1 button-only screens and schema-v2 text-only forms keep their
existing payloads and callbacks.

## Checkbox settings forms

Use `client_screen::open_settings_form` when a screen combines checkboxes with optional text
inputs or a single-choice list. `ClientScreenSettingsFormDefinition::new` takes the base screen,
zero to four `ClientScreenTextField` values, an optional `ClientScreenSelectionList`, and one to
eight `ClientScreenCheckbox` values. `ClientScreenCheckbox::new(id, label, initial_checked)`
validates a checkbox's stable ID and literal label. The constructor validates the complete form
before it is encoded:

- Element IDs must use 1–64 lowercase ASCII ID bytes (`a-z`, `0-9`, `_.:/-`) and be unique across
  buttons, text fields, the list, and checkboxes. Selection option IDs must also be unique within
  their list.
- Checkbox labels and list labels/options must be 1–96 UTF-8 bytes and contain no control
  characters. Text fields keep their existing 1–96 byte labels and have a configured value limit
  from 1–128 UTF-8 bytes.
- A selection list, when present, must contain 1–32 options; its initial selection must name an
  option. The complete encoded screen must fit in 4,096 bytes.

This is schema v4. It keeps the schema-v1 screen prefix, followed by text fields, an optional list,
and checkboxes. Schemas v1, v2, and v3 retain their existing byte layouts. The callback remains the
owner-only `client.screen.form_submit` callback; it must be exported before the screen is opened.
It needs the existing `client.ui.screen` capability and does not use `events::subscribe`. On any
form-button click, including Cancel, `ClientScreenFormSubmit.fields` contains values in declaration
order: text fields first, the selection value if present next, then checkboxes. Text values are the
current input strings, a list value is its selected option ID (or an empty string when unselected),
and checkbox values are the strings `"true"` or `"false"`. The clicked button ID is available in
`submit.button_id`. A maximum-size settings form returns 13 values: four text fields, one list
selection, and eight checkboxes. Escape closes the screen without submitting.

```rust
use sjml::client_screen::{
    self, ClientScreenButton, ClientScreenCheckbox, ClientScreenDefinition,
    ClientScreenSelectionList, ClientScreenSelectionOption, ClientScreenSettingsFormDefinition,
    ClientScreenTextField,
};
use sjml::events::ClientScreenFormSubmit;
use sjml::{client_command, event, SjmlResult};

#[client_command(name = "settings.open")]
fn open_settings() -> SjmlResult<String> {
    let screen = ClientScreenDefinition::new(
        "preferences",
        "Preferences",
        vec!["Choose how this client behaves.".to_owned()],
        vec![
            ClientScreenButton::new("save", "Save")?,
            ClientScreenButton::new("cancel", "Cancel")?,
        ],
    )?;
    let settings = ClientScreenSettingsFormDefinition::new(
        screen,
        vec![ClientScreenTextField::new("nickname", "Nickname", "", 32)?],
        Some(ClientScreenSelectionList::new(
            "quality",
            "Quality",
            vec![
                ClientScreenSelectionOption::new("low", "Low")?,
                ClientScreenSelectionOption::new("high", "High")?,
            ],
            Some("high".to_owned()),
        )?),
        vec![
            ClientScreenCheckbox::new("sound", "Enable sound", true)?,
            ClientScreenCheckbox::new("notifications", "Show notifications", false)?,
        ],
    )?;
    client_screen::open_settings_form(&settings)?;
    Ok("Opened settings".to_owned())
}

#[event(name = "client.screen.form_submit")]
fn handle_settings(submit: ClientScreenFormSubmit) -> SjmlResult<()> {
    if submit.screen_id != "preferences" {
        return Ok(());
    }
    let value = |id: &str| {
        submit.fields.iter()
            .find(|field| field.id == id)
            .map(|field| field.value.as_str())
            .unwrap_or_default()
    };
    let nickname = value("nickname");
    let quality = value("quality");
    let sound_enabled = value("sound") == "true";
    let notifications_enabled = value("notifications") == "true";
    let _ = (nickname, quality, sound_enabled, notifications_enabled, &submit.button_id);
    Ok(())
}
```

The command takes no arguments. Declare it with `client.command`, then run
`/sjmlclient client_screen_demo screen settings.open` in Minecraft:

```toml
[[modules]]
id = "screen"
path = "modules/screen.wasm"
side = "client"
permissions = ["client.ui.screen", "client.command"]

[[modules.client_commands]]
name = "settings.open"
schema_version = 1
permission_level = 0
```

Grant the capabilities in `config/sjml-permissions.toml`:

```toml
[[grant]]
module = "client_screen_demo/screen"
capabilities = ["client.ui.screen", "client.command"]
```

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
