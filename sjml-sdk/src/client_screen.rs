//! Bounded, module-owned interactive screens on the physical client.

use crate::{ErrorCode, SjmlError, SjmlResult};

/// Schema version used by client screen definitions sent to the host.
pub const CLIENT_SCREEN_SCHEMA_VERSION: u32 = 1;
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
        encode_definition, ClientScreenButton, ClientScreenDefinition,
        MAX_CLIENT_SCREEN_PAYLOAD_BYTES,
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

    fn append_string(target: &mut Vec<u8>, value: &str) {
        target.extend_from_slice(&(value.len() as u32).to_le_bytes());
        target.extend_from_slice(value.as_bytes());
    }
}
