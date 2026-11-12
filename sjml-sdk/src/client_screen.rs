//! Bounded, module-owned interactive screens on the physical client.

use crate::{ErrorCode, SjmlError, SjmlResult};

/// Schema version used by client screen definitions sent to the host.
pub const CLIENT_SCREEN_SCHEMA_VERSION: u32 = 1;
/// Schema version used by client form definitions sent to the host.
pub const CLIENT_SCREEN_FORM_SCHEMA_VERSION: u32 = 2;
/// Maximum UTF-8 length of a screen or button ID.
pub const MAX_CLIENT_SCREEN_ID_BYTES: usize = 64;
/// Maximum UTF-8 length of a screen title.
pub const MAX_CLIENT_SCREEN_TITLE_BYTES: usize = 128;
/// Maximum number of body lines in one screen.
pub const MAX_CLIENT_SCREEN_BODY_LINES: usize = 6;
/// Maximum UTF-8 length of one body line.
pub const MAX_CLIENT_SCREEN_BODY_LINE_BYTES: usize = 160;
/// Maximum number of buttons in one screen.
pub const MAX_CLIENT_SCREEN_BUTTONS: usize = 4;
/// Maximum UTF-8 length of one button label.
pub const MAX_CLIENT_SCREEN_BUTTON_LABEL_BYTES: usize = 96;
/// Maximum number of text fields in one form screen.
pub const MAX_CLIENT_SCREEN_TEXT_FIELDS: usize = 4;
/// Maximum UTF-8 length of one form field ID and label.
pub const MAX_CLIENT_SCREEN_TEXT_FIELD_ID_BYTES: usize = MAX_CLIENT_SCREEN_ID_BYTES;
/// Maximum UTF-8 length of one form field label.
pub const MAX_CLIENT_SCREEN_TEXT_FIELD_LABEL_BYTES: usize = MAX_CLIENT_SCREEN_BUTTON_LABEL_BYTES;
/// Maximum configurable UTF-8 length of one form field value.
pub const MAX_CLIENT_SCREEN_TEXT_FIELD_VALUE_BYTES: u32 = 128;
/// Maximum encoded size of a complete screen definition.
pub const MAX_CLIENT_SCREEN_PAYLOAD_BYTES: usize = 4096;

/// One bounded action button displayed by a client screen.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientScreenButton {
    /// Stable lowercase ID returned with this button's click event.
    pub id: String,
    /// Literal text displayed on the button.
    pub label: String,
}

impl ClientScreenButton {
    /// Builds and validates one screen button.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> SjmlResult<Self> {
        let button = Self {
            id: id.into(),
            label: label.into(),
        };
        button.validate()?;
        Ok(button)
    }

    fn validate(&self) -> SjmlResult<()> {
        validate_id(&self.id, "button ID")?;
        validate_text(
            &self.label,
            1,
            MAX_CLIENT_SCREEN_BUTTON_LABEL_BYTES,
            "button label",
        )
    }
}

/// A literal, module-owned interactive screen definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientScreenDefinition {
    /// Stable lowercase ID used to route the screen's button actions to its owner.
    pub id: String,
    /// Literal title shown at the top of the screen.
    pub title: String,
    /// Literal body lines shown below the title.
    pub body: Vec<String>,
    /// One through four unique action buttons.
    pub buttons: Vec<ClientScreenButton>,
}

impl ClientScreenDefinition {
    /// Builds and validates a bounded screen definition.
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        body: Vec<String>,
        buttons: Vec<ClientScreenButton>,
    ) -> SjmlResult<Self> {
        let definition = Self {
            id: id.into(),
            title: title.into(),
            body,
            buttons,
        };
        definition.validate()?;
        Ok(definition)
    }

    fn validate(&self) -> SjmlResult<usize> {
        validate_id(&self.id, "screen ID")?;
        validate_text(
            &self.title,
            1,
            MAX_CLIENT_SCREEN_TITLE_BYTES,
            "screen title",
        )?;
        if self.body.len() > MAX_CLIENT_SCREEN_BODY_LINES {
            return Err(too_large(format!(
                "screen body must contain at most {MAX_CLIENT_SCREEN_BODY_LINES} lines"
            )));
        }
        for line in &self.body {
            validate_text(
                line,
                0,
                MAX_CLIENT_SCREEN_BODY_LINE_BYTES,
                "screen body line",
            )?;
        }
        if !(1..=MAX_CLIENT_SCREEN_BUTTONS).contains(&self.buttons.len()) {
            return Err(invalid(format!(
                "a screen must contain between 1 and {MAX_CLIENT_SCREEN_BUTTONS} buttons"
            )));
        }
        for (index, button) in self.buttons.iter().enumerate() {
            button.validate()?;
            if self.buttons[..index]
                .iter()
                .any(|previous| previous.id == button.id)
            {
                return Err(invalid("screen button IDs must be unique"));
            }
        }

        let mut size = 4_usize;
        size = checked_string_size(size, &self.id)?;
        size = checked_string_size(size, &self.title)?;
        size = size.checked_add(4).ok_or_else(payload_overflow)?;
        for line in &self.body {
            size = checked_string_size(size, line)?;
        }
        size = size.checked_add(4).ok_or_else(payload_overflow)?;
        for button in &self.buttons {
            size = checked_string_size(size, &button.id)?;
            size = checked_string_size(size, &button.label)?;
        }
        if size > MAX_CLIENT_SCREEN_PAYLOAD_BYTES {
            return Err(too_large(format!(
                "encoded screen definition exceeds {MAX_CLIENT_SCREEN_PAYLOAD_BYTES} bytes"
            )));
        }
        Ok(size)
    }
}

/// One bounded text input displayed on a form screen.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientScreenTextField {
    /// Stable lowercase key included in the submit callback.
    pub id: String,
    /// Literal label displayed above the input box.
    pub label: String,
    /// Literal initial value placed in the input box.
    pub initial_value: String,
    /// Maximum UTF-8 byte length accepted by the input box and returned by the host.
    pub max_utf8_bytes: u32,
}

impl ClientScreenTextField {
    /// Builds and validates one bounded text field.
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        initial_value: impl Into<String>,
        max_utf8_bytes: u32,
    ) -> SjmlResult<Self> {
        let field = Self {
            id: id.into(),
            label: label.into(),
            initial_value: initial_value.into(),
            max_utf8_bytes,
        };
        field.validate()?;
        Ok(field)
    }

    fn validate(&self) -> SjmlResult<()> {
        validate_id(&self.id, "text field ID")?;
        validate_text(
            &self.label,
            1,
            MAX_CLIENT_SCREEN_TEXT_FIELD_LABEL_BYTES,
            "text field label",
        )?;
        if !(1..=MAX_CLIENT_SCREEN_TEXT_FIELD_VALUE_BYTES).contains(&self.max_utf8_bytes) {
            return Err(invalid(format!(
                "text field max_utf8_bytes must be 1–{MAX_CLIENT_SCREEN_TEXT_FIELD_VALUE_BYTES}"
            )));
        }
        validate_text(
            &self.initial_value,
            0,
            self.max_utf8_bytes as usize,
            "text field initial value",
        )
    }
}

/// A bounded form built on a screen definition and one to four text inputs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientScreenFormDefinition {
    /// Screen content and submit buttons.
    pub screen: ClientScreenDefinition,
    /// One to four uniquely identified text input fields.
    pub fields: Vec<ClientScreenTextField>,
}

impl ClientScreenFormDefinition {
    /// Builds and validates a bounded form screen.
    pub fn new(
        screen: ClientScreenDefinition,
        fields: Vec<ClientScreenTextField>,
    ) -> SjmlResult<Self> {
        let definition = Self { screen, fields };
        definition.validate()?;
        Ok(definition)
    }

    fn validate(&self) -> SjmlResult<usize> {
        let mut size = self.screen.validate()?;
        if !(1..=MAX_CLIENT_SCREEN_TEXT_FIELDS).contains(&self.fields.len()) {
            return Err(invalid(format!(
                "a form must contain between 1 and {MAX_CLIENT_SCREEN_TEXT_FIELDS} text fields"
            )));
        }
        for (index, field) in self.fields.iter().enumerate() {
            field.validate()?;
            if self.fields[..index]
                .iter()
                .any(|previous| previous.id == field.id)
                || self
                    .screen
                    .buttons
                    .iter()
                    .any(|button| button.id == field.id)
            {
                return Err(invalid(
                    "form field IDs must be unique and cannot match a button ID",
                ));
            }
            size = checked_string_size(size, &field.id)?;
            size = checked_string_size(size, &field.label)?;
            size = checked_string_size(size, &field.initial_value)?;
            size = size.checked_add(4).ok_or_else(payload_overflow)?;
        }
        size = size.checked_add(4).ok_or_else(payload_overflow)?;
        if size > MAX_CLIENT_SCREEN_PAYLOAD_BYTES {
            return Err(too_large(format!(
                "encoded client form exceeds {MAX_CLIENT_SCREEN_PAYLOAD_BYTES} bytes"
            )));
        }
        Ok(size)
    }
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_client_ui_v1")]
extern "C" {
    #[link_name = "open_screen"]
    fn imported_open_screen(payload_pointer: i32, payload_length: i32) -> i32;
}

/// Opens the screen on the local physical client.
///
/// The screen is literal data with a stable module-owned ID. Button clicks are later dispatched
/// only to that owning module as `client.screen.action`; the event carries the screen and button
/// IDs and cannot cancel the click. The bounded payload is copied synchronously by the host.
pub fn open(definition: &ClientScreenDefinition) -> SjmlResult<()> {
    let payload = encode_definition(definition)?;
    open_payload(&payload)
}

/// Opens a bounded text form on the local physical client.
///
/// The clicked button and the current value of every field are sent directly to the owning
/// module as `client.screen.form_submit`. The host copies the bounded payload synchronously.
pub fn open_form(definition: &ClientScreenFormDefinition) -> SjmlResult<()> {
    let payload = encode_form_definition(definition)?;
    open_payload(&payload)
}

fn open_payload(payload: &[u8]) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the bounded payload remains live while the host synchronously copies it.
        let status = unsafe {
            imported_open_screen(
                payload.as_ptr() as usize as u32 as i32,
                payload.len() as u32 as i32,
            )
        };
        if status == 0 {
            Ok(())
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected the client screen definition",
            ))
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = payload;
        Err(SjmlError::new(
            ErrorCode::UnknownErrorCode,
            "interactive screens require the wasm32-unknown-unknown target",
        ))
    }
}

fn encode_definition(definition: &ClientScreenDefinition) -> SjmlResult<Vec<u8>> {
    let capacity = definition.validate()?;
    let mut payload = Vec::with_capacity(capacity);
    write_u32(&mut payload, CLIENT_SCREEN_SCHEMA_VERSION);
    write_string(&mut payload, &definition.id)?;
    write_string(&mut payload, &definition.title)?;
    write_u32(&mut payload, definition.body.len() as u32);
    for line in &definition.body {
        write_string(&mut payload, line)?;
    }
    write_u32(&mut payload, definition.buttons.len() as u32);
    for button in &definition.buttons {
        write_string(&mut payload, &button.id)?;
        write_string(&mut payload, &button.label)?;
    }
    debug_assert!(payload.len() <= MAX_CLIENT_SCREEN_PAYLOAD_BYTES);
    Ok(payload)
}

fn encode_form_definition(definition: &ClientScreenFormDefinition) -> SjmlResult<Vec<u8>> {
    let capacity = definition.validate()?;
    let mut payload = encode_definition(&definition.screen)?;
    payload.reserve(capacity.saturating_sub(payload.len()));
    payload[..4].copy_from_slice(&CLIENT_SCREEN_FORM_SCHEMA_VERSION.to_le_bytes());
    write_u32(&mut payload, definition.fields.len() as u32);
    for field in &definition.fields {
        write_string(&mut payload, &field.id)?;
        write_string(&mut payload, &field.label)?;
        write_string(&mut payload, &field.initial_value)?;
        write_u32(&mut payload, field.max_utf8_bytes);
    }
    debug_assert!(payload.len() <= MAX_CLIENT_SCREEN_PAYLOAD_BYTES);
    Ok(payload)
}

fn write_string(payload: &mut Vec<u8>, value: &str) -> SjmlResult<()> {
    let length = u32::try_from(value.len()).map_err(|_| payload_overflow())?;
    write_u32(payload, length);
    payload.extend_from_slice(value.as_bytes());
    Ok(())
}

fn write_u32(payload: &mut Vec<u8>, value: u32) {
    payload.extend_from_slice(&value.to_le_bytes());
}

fn checked_string_size(size: usize, value: &str) -> SjmlResult<usize> {
    size.checked_add(4)
        .and_then(|size| size.checked_add(value.len()))
        .ok_or_else(payload_overflow)
}

fn validate_id(value: &str, name: &str) -> SjmlResult<()> {
    if value.is_empty()
        || value.len() > MAX_CLIENT_SCREEN_ID_BYTES
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_.:/-".contains(&byte)
        })
    {
        return Err(invalid(format!(
            "{name} must use 1–{MAX_CLIENT_SCREEN_ID_BYTES} lowercase ASCII ID bytes"
        )));
    }
    Ok(())
}

fn validate_text(value: &str, minimum: usize, maximum: usize, name: &str) -> SjmlResult<()> {
    if value.len() < minimum || value.len() > maximum {
        return Err(invalid(format!(
            "{name} must contain {minimum}–{maximum} UTF-8 bytes"
        )));
    }
    if value.chars().any(char::is_control) {
        return Err(invalid(format!(
            "{name} must not contain control characters"
        )));
    }
    Ok(())
}

fn invalid(message: impl Into<String>) -> SjmlError {
    SjmlError::new(ErrorCode::InvalidArgument, message)
}

fn too_large(message: impl Into<String>) -> SjmlError {
    SjmlError::new(ErrorCode::PayloadTooLarge, message)
}

fn payload_overflow() -> SjmlError {
    too_large("encoded client screen payload exceeds the supported size")
}

#[cfg(test)]
mod tests {
    use super::{
        encode_definition, encode_form_definition, ClientScreenButton, ClientScreenDefinition,
        ClientScreenFormDefinition, ClientScreenTextField, MAX_CLIENT_SCREEN_PAYLOAD_BYTES,
    };

    #[test]
    fn encodes_schema_v1_fields_in_the_documented_order() {
        let definition = ClientScreenDefinition::new(
            "demo:confirm",
            "Confirm",
            vec!["Continue?".to_owned(), String::new()],
            vec![
                ClientScreenButton::new("yes", "Yes").unwrap(),
                ClientScreenButton::new("no", "No").unwrap(),
            ],
        )
        .unwrap();
        let encoded = encode_definition(&definition).unwrap();
        let mut expected = 1_u32.to_le_bytes().to_vec();
        append_string(&mut expected, "demo:confirm");
        append_string(&mut expected, "Confirm");
        expected.extend_from_slice(&2_u32.to_le_bytes());
        append_string(&mut expected, "Continue?");
        append_string(&mut expected, "");
        expected.extend_from_slice(&2_u32.to_le_bytes());
        append_string(&mut expected, "yes");
        append_string(&mut expected, "Yes");
        append_string(&mut expected, "no");
        append_string(&mut expected, "No");
        assert_eq!(encoded, expected);
        assert!(encoded.len() <= MAX_CLIENT_SCREEN_PAYLOAD_BYTES);
    }

    #[test]
    fn encodes_schema_v2_form_fields_after_the_unchanged_screen_prefix() {
        let screen = ClientScreenDefinition::new(
            "demo:profile",
            "Profile",
            vec!["Enter your name".to_owned()],
            vec![ClientScreenButton::new("save", "Save").unwrap()],
        )
        .unwrap();
        let form = ClientScreenFormDefinition::new(
            screen.clone(),
            vec![ClientScreenTextField::new("name", "Name", "Ada", 64).unwrap()],
        )
        .unwrap();
        let encoded = encode_form_definition(&form).unwrap();
        let mut expected = encode_definition(&screen).unwrap();
        expected[..4].copy_from_slice(&2_u32.to_le_bytes());
        expected.extend_from_slice(&1_u32.to_le_bytes());
        append_string(&mut expected, "name");
        append_string(&mut expected, "Name");
        append_string(&mut expected, "Ada");
        expected.extend_from_slice(&64_u32.to_le_bytes());
        assert_eq!(encoded, expected);
        assert!(encoded.len() <= MAX_CLIENT_SCREEN_PAYLOAD_BYTES);
    }

    #[test]
    fn rejects_duplicate_or_button_colliding_form_field_ids() {
        let screen = ClientScreenDefinition::new(
            "demo:profile",
            "Profile",
            Vec::new(),
            vec![ClientScreenButton::new("save", "Save").unwrap()],
        )
        .unwrap();
        let field = || ClientScreenTextField::new("name", "Name", "", 32).unwrap();
        assert!(ClientScreenFormDefinition::new(screen.clone(), vec![field(), field()]).is_err());
        assert!(ClientScreenFormDefinition::new(
            screen,
            vec![ClientScreenTextField::new("save", "Name", "", 32).unwrap()],
        )
        .is_err());
    }

    fn append_string(target: &mut Vec<u8>, value: &str) {
        target.extend_from_slice(&(value.len() as u32).to_le_bytes());
        target.extend_from_slice(value.as_bytes());
    }
}
