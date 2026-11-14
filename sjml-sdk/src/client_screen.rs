//! Bounded, module-owned interactive screens on the physical client.

#[cfg(target_arch = "wasm32")]
use crate::abi::{allocate_host_input, deallocate_host_input};
use crate::{ErrorCode, SjmlError, SjmlResult};

/// Schema version used by client screen definitions sent to the host.
pub const CLIENT_SCREEN_SCHEMA_VERSION: u32 = 1;
/// Schema version used by client form definitions sent to the host.
pub const CLIENT_SCREEN_FORM_SCHEMA_VERSION: u32 = 2;
/// Schema version used by client list form definitions sent to the host.
pub const CLIENT_SCREEN_LIST_FORM_SCHEMA_VERSION: u32 = 3;
/// Schema version used by checkbox settings forms sent to the host.
pub const CLIENT_SCREEN_SETTINGS_FORM_SCHEMA_VERSION: u32 = 4;
/// Schema version used by settings forms with one integer slider.
pub const CLIENT_SCREEN_SLIDER_FORM_SCHEMA_VERSION: u32 = 5;
/// Schema version used by bounded multi-select list forms.
pub const CLIENT_SCREEN_MULTI_SELECT_FORM_SCHEMA_VERSION: u32 = 6;
/// Schema version for the owner-scoped update envelope around one complete screen definition.
pub const CLIENT_SCREEN_UPDATE_SCHEMA_VERSION: u32 = 7;
/// Schema version for adding passive progress bars to any existing screen/form definition.
pub const CLIENT_SCREEN_PROGRESS_BAR_SCHEMA_VERSION: u32 = 8;
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
/// Maximum number of options in a client selection list.
pub const MAX_CLIENT_SCREEN_SELECTION_OPTIONS: usize = 32;
/// Maximum number of options and initially selected IDs in a multi-select list.
pub const MAX_CLIENT_SCREEN_MULTI_SELECT_OPTIONS: usize = 32;
/// Maximum number of checkbox fields in a client settings form.
pub const MAX_CLIENT_SCREEN_CHECKBOXES: usize = 8;
/// Absolute bound accepted by client numeric slider endpoints and values.
pub const MAX_CLIENT_SCREEN_SLIDER_ABS_VALUE: i32 = 1_000_000;
/// Maximum number of passive progress bars on one screen.
pub const MAX_CLIENT_SCREEN_PROGRESS_BARS: usize = 4;
/// Absolute bound accepted by progress-bar range endpoints and values.
pub const MAX_CLIENT_SCREEN_PROGRESS_BAR_ABS_VALUE: i32 = 1_000_000;
/// Maximum UTF-8 length of one selection list or option label.
pub const MAX_CLIENT_SCREEN_SELECTION_LABEL_BYTES: usize = 96;
/// Maximum encoded size of a complete screen definition.
pub const MAX_CLIENT_SCREEN_PAYLOAD_BYTES: usize = 4096;
#[cfg(target_arch = "wasm32")]
const CLOSE_RESULT_BYTES: u32 = 1;

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

/// One literal option in a bounded client selection list.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientScreenSelectionOption {
    /// Stable lowercase option ID returned with the selected value.
    pub id: String,
    /// Literal label shown for this option.
    pub label: String,
}

impl ClientScreenSelectionOption {
    /// Builds and validates one selection option.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> SjmlResult<Self> {
        let option = Self {
            id: id.into(),
            label: label.into(),
        };
        option.validate()?;
        Ok(option)
    }

    fn validate(&self) -> SjmlResult<()> {
        validate_id(&self.id, "selection option ID")?;
        validate_text(
            &self.label,
            1,
            MAX_CLIENT_SCREEN_SELECTION_LABEL_BYTES,
            "selection option label",
        )
    }
}

/// One bounded, single-choice input displayed on a list form screen.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientScreenSelectionList {
    /// Stable lowercase key included in the submit callback.
    pub id: String,
    /// Literal label displayed above the selection control.
    pub label: String,
    /// One through 32 literal choices.
    pub options: Vec<ClientScreenSelectionOption>,
    /// Initially selected option ID, or `None` to start without a selection.
    pub initial_selected_id: Option<String>,
}

impl ClientScreenSelectionList {
    /// Builds and validates one bounded selection list.
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        options: Vec<ClientScreenSelectionOption>,
        initial_selected_id: Option<String>,
    ) -> SjmlResult<Self> {
        let list = Self {
            id: id.into(),
            label: label.into(),
            options,
            initial_selected_id,
        };
        list.validate()?;
        Ok(list)
    }

    fn validate(&self) -> SjmlResult<()> {
        validate_id(&self.id, "selection list ID")?;
        validate_text(
            &self.label,
            1,
            MAX_CLIENT_SCREEN_SELECTION_LABEL_BYTES,
            "selection list label",
        )?;
        if !(1..=MAX_CLIENT_SCREEN_SELECTION_OPTIONS).contains(&self.options.len()) {
            return Err(invalid(format!(
                "a selection list must contain between 1 and {MAX_CLIENT_SCREEN_SELECTION_OPTIONS} options"
            )));
        }
        for (index, option) in self.options.iter().enumerate() {
            option.validate()?;
            if self.options[..index]
                .iter()
                .any(|previous| previous.id == option.id)
            {
                return Err(invalid("selection option IDs must be unique"));
            }
        }
        if let Some(selected_id) = &self.initial_selected_id {
            validate_id(selected_id, "initial selected option ID")?;
            if !self.options.iter().any(|option| option.id == *selected_id) {
                return Err(invalid(
                    "initial selected option ID must reference a selection option",
                ));
            }
        }
        Ok(())
    }
}

/// A bounded form screen with zero to four text inputs and one selection list.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientScreenListFormDefinition {
    /// Screen content and submit buttons.
    pub screen: ClientScreenDefinition,
    /// Zero to four uniquely identified text input fields.
    pub fields: Vec<ClientScreenTextField>,
    /// The form's single bounded selection list.
    pub list: ClientScreenSelectionList,
}

impl ClientScreenListFormDefinition {
    /// Builds and validates a bounded list form screen.
    pub fn new(
        screen: ClientScreenDefinition,
        fields: Vec<ClientScreenTextField>,
        list: ClientScreenSelectionList,
    ) -> SjmlResult<Self> {
        let definition = Self {
            screen,
            fields,
            list,
        };
        definition.validate()?;
        Ok(definition)
    }

    fn validate(&self) -> SjmlResult<usize> {
        let mut size = self.screen.validate()?;
        self.list.validate()?;
        if self.fields.len() > MAX_CLIENT_SCREEN_TEXT_FIELDS {
            return Err(invalid(format!(
                "a list form supports at most {MAX_CLIENT_SCREEN_TEXT_FIELDS} text fields"
            )));
        }
        if self
            .screen
            .buttons
            .iter()
            .any(|button| button.id == self.list.id)
            || self.fields.iter().any(|field| field.id == self.list.id)
        {
            return Err(invalid(
                "selection list ID must be unique and cannot match a button ID",
            ));
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
        }
        size = size.checked_add(4).ok_or_else(payload_overflow)?;
        for field in &self.fields {
            size = checked_string_size(size, &field.id)?;
            size = checked_string_size(size, &field.label)?;
            size = checked_string_size(size, &field.initial_value)?;
            size = size.checked_add(4).ok_or_else(payload_overflow)?;
        }
        size = size.checked_add(4).ok_or_else(payload_overflow)?;
        size = checked_string_size(size, &self.list.id)?;
        size = checked_string_size(size, &self.list.label)?;
        size = checked_string_size(size, self.list.initial_selected_id.as_deref().unwrap_or(""))?;
        size = size.checked_add(4).ok_or_else(payload_overflow)?;
        for option in &self.list.options {
            size = checked_string_size(size, &option.id)?;
            size = checked_string_size(size, &option.label)?;
        }
        if size > MAX_CLIENT_SCREEN_PAYLOAD_BYTES {
            return Err(too_large(format!(
                "encoded client list form exceeds {MAX_CLIENT_SCREEN_PAYLOAD_BYTES} bytes"
            )));
        }
        Ok(size)
    }
}

/// One bounded multi-select list displayed on a schema-v6 form screen.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientScreenMultiSelectionList {
    /// Stable lowercase key included in the submit callback.
    pub id: String,
    /// Literal label displayed above the list.
    pub label: String,
    /// One through 32 literal choices.
    pub options: Vec<ClientScreenSelectionOption>,
    /// Initially selected option IDs, which must refer to distinct options.
    pub initial_selected_ids: Vec<String>,
}

impl ClientScreenMultiSelectionList {
    /// Builds and validates one bounded multi-select list.
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        options: Vec<ClientScreenSelectionOption>,
        initial_selected_ids: Vec<String>,
    ) -> SjmlResult<Self> {
        let list = Self {
            id: id.into(),
            label: label.into(),
            options,
            initial_selected_ids,
        };
        list.validate()?;
        Ok(list)
    }

    fn validate(&self) -> SjmlResult<()> {
        validate_id(&self.id, "multi-selection list ID")?;
        validate_text(
            &self.label,
            1,
            MAX_CLIENT_SCREEN_SELECTION_LABEL_BYTES,
            "multi-selection list label",
        )?;
        if !(1..=MAX_CLIENT_SCREEN_MULTI_SELECT_OPTIONS).contains(&self.options.len()) {
            return Err(invalid(format!(
                "a multi-selection list must contain between 1 and {MAX_CLIENT_SCREEN_MULTI_SELECT_OPTIONS} options"
            )));
        }
        for (index, option) in self.options.iter().enumerate() {
            option.validate()?;
            if self.options[..index]
                .iter()
                .any(|previous| previous.id == option.id)
            {
                return Err(invalid("multi-selection option IDs must be unique"));
            }
        }
        if self.initial_selected_ids.len() > MAX_CLIENT_SCREEN_MULTI_SELECT_OPTIONS {
            return Err(too_large(format!(
                "a multi-selection list supports at most {MAX_CLIENT_SCREEN_MULTI_SELECT_OPTIONS} initially selected options"
            )));
        }
        for (index, selected_id) in self.initial_selected_ids.iter().enumerate() {
            validate_id(selected_id, "initial selected option ID")?;
            if self.initial_selected_ids[..index]
                .iter()
                .any(|previous| previous == selected_id)
                || !self.options.iter().any(|option| option.id == *selected_id)
            {
                return Err(invalid(
                    "initial selected option IDs must be unique IDs from the multi-selection list",
                ));
            }
        }
        Ok(())
    }
}

/// A schema-v6 form with zero to four text fields and one multi-select list.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientScreenMultiSelectFormDefinition {
    /// Screen content and submit buttons.
    pub screen: ClientScreenDefinition,
    /// Zero to four uniquely identified text inputs.
    pub fields: Vec<ClientScreenTextField>,
    /// The form's bounded multi-select list.
    pub list: ClientScreenMultiSelectionList,
}

impl ClientScreenMultiSelectFormDefinition {
    /// Builds and validates a bounded schema-v6 multi-select form.
    pub fn new(
        screen: ClientScreenDefinition,
        fields: Vec<ClientScreenTextField>,
        list: ClientScreenMultiSelectionList,
    ) -> SjmlResult<Self> {
        let definition = Self {
            screen,
            fields,
            list,
        };
        definition.validate()?;
        Ok(definition)
    }

    fn validate(&self) -> SjmlResult<usize> {
        let mut size = self.screen.validate()?;
        self.list.validate()?;
        if self.fields.len() > MAX_CLIENT_SCREEN_TEXT_FIELDS {
            return Err(invalid(format!(
                "a multi-select form supports at most {MAX_CLIENT_SCREEN_TEXT_FIELDS} text fields"
            )));
        }
        let mut ids: Vec<String> = self
            .screen
            .buttons
            .iter()
            .map(|button| button.id.clone())
            .collect();
        for field in &self.fields {
            field.validate()?;
            if ids.contains(&field.id) {
                return Err(invalid(
                    "multi-select form element IDs must be unique and cannot match a button ID",
                ));
            }
            ids.push(field.id.clone());
        }
        if ids.contains(&self.list.id) {
            return Err(invalid(
                "multi-selection list ID must be unique and cannot match a button or field ID",
            ));
        }
        size = size.checked_add(4).ok_or_else(payload_overflow)?;
        for field in &self.fields {
            size = checked_string_size(size, &field.id)?;
            size = checked_string_size(size, &field.label)?;
            size = checked_string_size(size, &field.initial_value)?;
            size = size.checked_add(4).ok_or_else(payload_overflow)?;
        }
        size = size.checked_add(4).ok_or_else(payload_overflow)?;
        size = checked_string_size(size, &self.list.id)?;
        size = checked_string_size(size, &self.list.label)?;
        size = size.checked_add(4).ok_or_else(payload_overflow)?;
        for selected_id in &self.list.initial_selected_ids {
            size = checked_string_size(size, selected_id)?;
        }
        size = size.checked_add(4).ok_or_else(payload_overflow)?;
        for option in &self.list.options {
            size = checked_string_size(size, &option.id)?;
            size = checked_string_size(size, &option.label)?;
        }
        if size > MAX_CLIENT_SCREEN_PAYLOAD_BYTES {
            return Err(too_large(format!(
                "encoded client multi-select form exceeds {MAX_CLIENT_SCREEN_PAYLOAD_BYTES} bytes"
            )));
        }
        Ok(size)
    }
}

/// A passive progress bar displayed with a screen or form.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientScreenProgressBar {
    /// Literal label displayed beside the current value.
    pub label: String,
    /// Inclusive minimum value represented by an empty bar.
    pub minimum: i32,
    /// Inclusive maximum value represented by a full bar.
    pub maximum: i32,
    /// Current value, which must lie inside the inclusive range.
    pub value: i32,
}

impl ClientScreenProgressBar {
    /// Builds and validates one passive progress bar.
    pub fn new(
        label: impl Into<String>,
        minimum: i32,
        maximum: i32,
        value: i32,
    ) -> SjmlResult<Self> {
        let progress = Self {
            label: label.into(),
            minimum,
            maximum,
            value,
        };
        progress.validate()?;
        Ok(progress)
    }

    /// Returns the normalized fill fraction in the inclusive range 0.0–1.0.
    pub fn fraction(&self) -> f32 {
        if self.maximum <= self.minimum || self.value <= self.minimum {
            return 0.0;
        }
        if self.value >= self.maximum {
            return 1.0;
        }
        let raw = (f64::from(self.value) - f64::from(self.minimum))
            / (f64::from(self.maximum) - f64::from(self.minimum));
        raw.clamp(0.0, 1.0) as f32
    }

    fn validate(&self) -> SjmlResult<()> {
        validate_text(
            &self.label,
            1,
            MAX_CLIENT_SCREEN_BUTTON_LABEL_BYTES,
            "progress bar label",
        )?;
        if self.minimum < -MAX_CLIENT_SCREEN_PROGRESS_BAR_ABS_VALUE
            || self.maximum > MAX_CLIENT_SCREEN_PROGRESS_BAR_ABS_VALUE
            || self.minimum >= self.maximum
            || self.value < self.minimum
            || self.value > self.maximum
        {
            return Err(invalid(
                "progress bar requires ordered bounds within ±1000000 and a value inside those bounds",
            ));
        }
        Ok(())
    }
}

/// Implemented by SDK screen/form definitions that can be decorated with passive widgets.
#[doc(hidden)]
pub trait ClientScreenDefinitionSource {
    #[doc(hidden)]
    fn encode_client_screen_definition(&self) -> SjmlResult<Vec<u8>>;
}

/// Existing screen or form content plus one to four passive progress bars.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientScreenWithProgressBars<D: ClientScreenDefinitionSource> {
    /// The base screen, text form, list form, settings form, slider form, or multi-select form.
    pub definition: D,
    /// One to four progress bars, displayed below the body and above form controls.
    pub progress_bars: Vec<ClientScreenProgressBar>,
}

impl<D: ClientScreenDefinitionSource> ClientScreenWithProgressBars<D> {
    /// Builds and validates a screen/form with one to four passive progress bars.
    pub fn new(definition: D, progress_bars: Vec<ClientScreenProgressBar>) -> SjmlResult<Self> {
        if !(1..=MAX_CLIENT_SCREEN_PROGRESS_BARS).contains(&progress_bars.len()) {
            return Err(invalid(format!(
                "a progress screen must contain between 1 and {MAX_CLIENT_SCREEN_PROGRESS_BARS} bars"
            )));
        }
        for progress_bar in &progress_bars {
            progress_bar.validate()?;
        }
        let result = Self {
            definition,
            progress_bars,
        };
        let _ = result.encode_definition()?;
        Ok(result)
    }

    /// Opens this screen/form with its passive progress bars on the physical client.
    pub fn open(&self) -> SjmlResult<()> {
        open_payload(&self.encode_definition()?)
    }

    /// Replaces the current same-ID owner screen/form and its progress bars in place.
    pub fn update(&self) -> SjmlResult<()> {
        update_payload(&self.encode_definition()?)
    }

    fn encode_definition(&self) -> SjmlResult<Vec<u8>> {
        let base = self.definition.encode_client_screen_definition()?;
        encode_progress_bars_definition(&base, &self.progress_bars)
    }
}

/// One literal boolean input displayed on a settings form screen.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientScreenCheckbox {
    /// Stable lowercase key included in the submit callback.
    pub id: String,
    /// Literal label displayed beside the checkbox.
    pub label: String,
    /// Initial checked state.
    pub initial_checked: bool,
}

/// One bounded integer slider submitted as a canonical decimal string.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientScreenNumericSlider {
    /// Stable lowercase key included in the submit callback.
    pub id: String,
    /// Literal label displayed above the slider.
    pub label: String,
    /// Inclusive minimum value.
    pub minimum: i32,
    /// Inclusive maximum value.
    pub maximum: i32,
    /// Positive distance between adjacent values.
    pub step: i32,
    /// Initial value, which must align with the range and step.
    pub initial_value: i32,
}

impl ClientScreenNumericSlider {
    /// Builds and validates one bounded integer slider.
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        minimum: i32,
        maximum: i32,
        step: i32,
        initial_value: i32,
    ) -> SjmlResult<Self> {
        let slider = Self {
            id: id.into(),
            label: label.into(),
            minimum,
            maximum,
            step,
            initial_value,
        };
        slider.validate()?;
        Ok(slider)
    }

    fn validate(&self) -> SjmlResult<()> {
        validate_id(&self.id, "numeric slider ID")?;
        validate_text(
            &self.label,
            1,
            MAX_CLIENT_SCREEN_SELECTION_LABEL_BYTES,
            "numeric slider label",
        )?;
        let span = i64::from(self.maximum) - i64::from(self.minimum);
        if self.minimum < -MAX_CLIENT_SCREEN_SLIDER_ABS_VALUE
            || self.maximum > MAX_CLIENT_SCREEN_SLIDER_ABS_VALUE
            || self.minimum >= self.maximum
            || self.step < 1
            || span % i64::from(self.step) != 0
            || self.initial_value < self.minimum
            || self.initial_value > self.maximum
            || (i64::from(self.initial_value) - i64::from(self.minimum)) % i64::from(self.step) != 0
        {
            return Err(invalid(
                "numeric slider must use aligned bounds within ±1000000 and an aligned initial value",
            ));
        }
        Ok(())
    }
}

/// A form with optional text, list, and checkbox controls plus one numeric slider.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientScreenSliderFormDefinition {
    /// Screen content and submit buttons.
    pub screen: ClientScreenDefinition,
    /// Zero to four uniquely identified text input fields.
    pub fields: Vec<ClientScreenTextField>,
    /// Optional single-choice list.
    pub list: Option<ClientScreenSelectionList>,
    /// Zero to eight uniquely identified checkbox fields.
    pub checkboxes: Vec<ClientScreenCheckbox>,
    /// The required bounded integer slider.
    pub slider: ClientScreenNumericSlider,
}

impl ClientScreenSliderFormDefinition {
    /// Builds and validates a bounded slider form.
    pub fn new(
        screen: ClientScreenDefinition,
        fields: Vec<ClientScreenTextField>,
        list: Option<ClientScreenSelectionList>,
        checkboxes: Vec<ClientScreenCheckbox>,
        slider: ClientScreenNumericSlider,
    ) -> SjmlResult<Self> {
        let definition = Self {
            screen,
            fields,
            list,
            checkboxes,
            slider,
        };
        definition.validate()?;
        Ok(definition)
    }

    fn validate(&self) -> SjmlResult<usize> {
        let mut size = self.screen.validate()?;
        self.slider.validate()?;
        if self.fields.len() > MAX_CLIENT_SCREEN_TEXT_FIELDS {
            return Err(invalid(format!(
                "a slider form supports at most {MAX_CLIENT_SCREEN_TEXT_FIELDS} text fields"
            )));
        }
        if self.checkboxes.len() > MAX_CLIENT_SCREEN_CHECKBOXES {
            return Err(invalid(format!(
                "a slider form supports at most {MAX_CLIENT_SCREEN_CHECKBOXES} checkboxes"
            )));
        }
        let mut ids: Vec<String> = self
            .screen
            .buttons
            .iter()
            .map(|button| button.id.clone())
            .collect();
        for field in &self.fields {
            field.validate()?;
            if ids.contains(&field.id) {
                return Err(invalid(
                    "slider form element IDs must be unique and cannot match a button ID",
                ));
            }
            ids.push(field.id.clone());
        }
        if let Some(list) = &self.list {
            list.validate()?;
            if ids.contains(&list.id) {
                return Err(invalid(
                    "slider form element IDs must be unique and cannot match a button ID",
                ));
            }
            ids.push(list.id.clone());
        }
        for checkbox in &self.checkboxes {
            checkbox.validate()?;
            if ids.contains(&checkbox.id) {
                return Err(invalid(
                    "slider form element IDs must be unique and cannot match a button ID",
                ));
            }
            ids.push(checkbox.id.clone());
        }
        if ids.contains(&self.slider.id) {
            return Err(invalid(
                "slider form element IDs must be unique and cannot match a button ID",
            ));
        }

        size = size.checked_add(4).ok_or_else(payload_overflow)?;
        for field in &self.fields {
            size = checked_string_size(size, &field.id)?;
            size = checked_string_size(size, &field.label)?;
            size = checked_string_size(size, &field.initial_value)?;
            size = size.checked_add(4).ok_or_else(payload_overflow)?;
        }
        size = size.checked_add(4).ok_or_else(payload_overflow)?;
        if let Some(list) = &self.list {
            size = checked_string_size(size, &list.id)?;
            size = checked_string_size(size, &list.label)?;
            size = checked_string_size(size, list.initial_selected_id.as_deref().unwrap_or(""))?;
            size = size.checked_add(4).ok_or_else(payload_overflow)?;
            for option in &list.options {
                size = checked_string_size(size, &option.id)?;
                size = checked_string_size(size, &option.label)?;
            }
        }
        size = size.checked_add(4).ok_or_else(payload_overflow)?;
        for checkbox in &self.checkboxes {
            size = checked_string_size(size, &checkbox.id)?;
            size = checked_string_size(size, &checkbox.label)?;
            size = size.checked_add(4).ok_or_else(payload_overflow)?;
        }
        size = checked_string_size(size, &self.slider.id)?;
        size = checked_string_size(size, &self.slider.label)?;
        size = size.checked_add(4 * 4).ok_or_else(payload_overflow)?;
        if size > MAX_CLIENT_SCREEN_PAYLOAD_BYTES {
            return Err(too_large(format!(
                "encoded client slider form exceeds {MAX_CLIENT_SCREEN_PAYLOAD_BYTES} bytes"
            )));
        }
        Ok(size)
    }
}

impl ClientScreenCheckbox {
    /// Builds and validates one checkbox field.
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        initial_checked: bool,
    ) -> SjmlResult<Self> {
        let checkbox = Self {
            id: id.into(),
            label: label.into(),
            initial_checked,
        };
        checkbox.validate()?;
        Ok(checkbox)
    }

    fn validate(&self) -> SjmlResult<()> {
        validate_id(&self.id, "checkbox ID")?;
        validate_text(
            &self.label,
            1,
            MAX_CLIENT_SCREEN_SELECTION_LABEL_BYTES,
            "checkbox label",
        )
    }
}

/// A settings form with zero to four text fields, an optional selection list, and one to eight checkboxes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientScreenSettingsFormDefinition {
    /// Screen content and submit buttons.
    pub screen: ClientScreenDefinition,
    /// Zero to four uniquely identified text input fields.
    pub fields: Vec<ClientScreenTextField>,
    /// Optional single-choice list.
    pub list: Option<ClientScreenSelectionList>,
    /// One to eight uniquely identified checkbox fields.
    pub checkboxes: Vec<ClientScreenCheckbox>,
}

impl ClientScreenSettingsFormDefinition {
    /// Builds and validates a bounded settings form screen.
    pub fn new(
        screen: ClientScreenDefinition,
        fields: Vec<ClientScreenTextField>,
        list: Option<ClientScreenSelectionList>,
        checkboxes: Vec<ClientScreenCheckbox>,
    ) -> SjmlResult<Self> {
        let definition = Self {
            screen,
            fields,
            list,
            checkboxes,
        };
        definition.validate()?;
        Ok(definition)
    }

    fn validate(&self) -> SjmlResult<usize> {
        let mut size = self.screen.validate()?;
        if self.fields.len() > MAX_CLIENT_SCREEN_TEXT_FIELDS {
            return Err(invalid(format!(
                "a settings form supports at most {MAX_CLIENT_SCREEN_TEXT_FIELDS} text fields"
            )));
        }
        if !(1..=MAX_CLIENT_SCREEN_CHECKBOXES).contains(&self.checkboxes.len()) {
            return Err(invalid(format!(
                "a settings form must contain between 1 and {MAX_CLIENT_SCREEN_CHECKBOXES} checkboxes"
            )));
        }

        let mut ids: Vec<String> = self
            .screen
            .buttons
            .iter()
            .map(|button| button.id.clone())
            .collect();
        for field in &self.fields {
            field.validate()?;
            if ids.contains(&field.id) {
                return Err(invalid(
                    "settings form element IDs must be unique and cannot match a button ID",
                ));
            }
            ids.push(field.id.clone());
        }
        if let Some(list) = &self.list {
            list.validate()?;
            if ids.contains(&list.id) {
                return Err(invalid(
                    "settings form element IDs must be unique and cannot match a button ID",
                ));
            }
            ids.push(list.id.clone());
        }
        for checkbox in &self.checkboxes {
            checkbox.validate()?;
            if ids.contains(&checkbox.id) {
                return Err(invalid(
                    "settings form element IDs must be unique and cannot match a button ID",
                ));
            }
            ids.push(checkbox.id.clone());
        }

        size = size.checked_add(4).ok_or_else(payload_overflow)?;
        for field in &self.fields {
            size = checked_string_size(size, &field.id)?;
            size = checked_string_size(size, &field.label)?;
            size = checked_string_size(size, &field.initial_value)?;
            size = size.checked_add(4).ok_or_else(payload_overflow)?;
        }
        size = size.checked_add(4).ok_or_else(payload_overflow)?;
        if let Some(list) = &self.list {
            size = checked_string_size(size, &list.id)?;
            size = checked_string_size(size, &list.label)?;
            size = checked_string_size(size, list.initial_selected_id.as_deref().unwrap_or(""))?;
            size = size.checked_add(4).ok_or_else(payload_overflow)?;
            for option in &list.options {
                size = checked_string_size(size, &option.id)?;
                size = checked_string_size(size, &option.label)?;
            }
        }
        size = size.checked_add(4).ok_or_else(payload_overflow)?;
        for checkbox in &self.checkboxes {
            size = checked_string_size(size, &checkbox.id)?;
            size = checked_string_size(size, &checkbox.label)?;
            size = size.checked_add(4).ok_or_else(payload_overflow)?;
        }
        if size > MAX_CLIENT_SCREEN_PAYLOAD_BYTES {
            return Err(too_large(format!(
                "encoded client settings form exceeds {MAX_CLIENT_SCREEN_PAYLOAD_BYTES} bytes"
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

    #[link_name = "update_screen"]
    fn imported_update_screen(payload_pointer: i32, payload_length: i32) -> i32;

    #[link_name = "close_screen"]
    fn imported_close_screen(
        screen_id_pointer: i32,
        screen_id_length: i32,
        result_pointer: i32,
        result_capacity: i32,
    ) -> i32;
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

/// Opens a bounded form with a single-choice list on the local physical client.
///
/// On a button click, the host sends all text values followed by the selected option (or an empty
/// value when no option is selected) directly to the owning module as `client.screen.form_submit`.
pub fn open_list_form(definition: &ClientScreenListFormDefinition) -> SjmlResult<()> {
    let payload = encode_list_form_definition(definition)?;
    open_payload(&payload)
}

/// Opens a settings form with optional text/list controls and one to eight checkboxes.
///
/// On submit, values follow declaration order: text fields, optional selection, then checkboxes.
/// Checkbox values use the strings `"true"` and `"false"` in `client.screen.form_submit`.
pub fn open_settings_form(definition: &ClientScreenSettingsFormDefinition) -> SjmlResult<()> {
    let payload = encode_settings_form_definition(definition)?;
    open_payload(&payload)
}

/// Opens a form with one bounded integer slider on the local physical client.
///
/// Values are local while dragging and are submitted as canonical decimal strings only when a
/// button is clicked. Submission order is text fields, optional selection, checkboxes, then slider.
pub fn open_slider_form(definition: &ClientScreenSliderFormDefinition) -> SjmlResult<()> {
    let payload = encode_slider_form_definition(definition)?;
    open_payload(&payload)
}

/// Opens a bounded multi-select form on the local physical client.
///
/// List toggles and scrolling stay local. A submit button sends text fields and the selected
/// option IDs directly to the owner through `client.screen.multi_select_form_submit`.
pub fn open_multi_select_form(
    definition: &ClientScreenMultiSelectFormDefinition,
) -> SjmlResult<()> {
    let payload = encode_multi_select_form_definition(definition)?;
    open_payload(&payload)
}

/// Updates this module's currently open screen without closing or replacing its session.
///
/// The definition must use the same stable screen ID passed to [`open`]. The host only updates a
/// screen still owned by this module; a missing, stale, or replaced screen returns an ABI error.
pub fn update(definition: &ClientScreenDefinition) -> SjmlResult<()> {
    update_payload(&encode_definition(definition)?)
}

/// Updates this module's active bounded text form in place. Its screen ID must match [`open_form`].
pub fn update_form(definition: &ClientScreenFormDefinition) -> SjmlResult<()> {
    update_payload(&encode_form_definition(definition)?)
}

/// Updates this module's active single-choice form in place. Its screen ID must match
/// [`open_list_form`].
pub fn update_list_form(definition: &ClientScreenListFormDefinition) -> SjmlResult<()> {
    update_payload(&encode_list_form_definition(definition)?)
}

/// Updates this module's active settings form in place. Its screen ID must match
/// [`open_settings_form`].
pub fn update_settings_form(definition: &ClientScreenSettingsFormDefinition) -> SjmlResult<()> {
    update_payload(&encode_settings_form_definition(definition)?)
}

/// Updates this module's active slider form in place. Its screen ID must match
/// [`open_slider_form`].
pub fn update_slider_form(definition: &ClientScreenSliderFormDefinition) -> SjmlResult<()> {
    update_payload(&encode_slider_form_definition(definition)?)
}

/// Updates this module's active multi-select form in place. Its screen ID must match
/// [`open_multi_select_form`].
pub fn update_multi_select_form(
    definition: &ClientScreenMultiSelectFormDefinition,
) -> SjmlResult<()> {
    update_payload(&encode_multi_select_form_definition(definition)?)
}

/// Closes this module's currently open screen when its stable ID still matches.
///
/// Returns `true` only when the host closed that exact screen. A stale screen ID or a screen
/// replaced by another UI returns `false`; the host never closes another module's or a vanilla
/// screen. The operation requires the same physical-client context and `client.ui.screen` grant
/// as [`open`].
pub fn close(screen_id: &str) -> SjmlResult<bool> {
    validate_id(screen_id, "screen ID")?;
    #[cfg(target_arch = "wasm32")]
    {
        let screen_id_bytes = screen_id.as_bytes();
        let screen_id_pointer = allocate_host_input(screen_id_bytes.len() as u32);
        let result_pointer = allocate_host_input(CLOSE_RESULT_BYTES);
        if screen_id_pointer == 0 || result_pointer == 0 {
            // SAFETY: each non-zero pointer is a live allocation with the exact size supplied.
            unsafe {
                deallocate_host_input(screen_id_pointer, screen_id_bytes.len() as u32);
                deallocate_host_input(result_pointer, CLOSE_RESULT_BYTES);
            }
            return Err(SjmlError::new(
                ErrorCode::UnknownErrorCode,
                "could not allocate client screen close buffers",
            ));
        }
        // SAFETY: both pointers refer to allocations with the documented live bounds.
        unsafe {
            core::ptr::copy_nonoverlapping(
                screen_id_bytes.as_ptr(),
                screen_id_pointer as *mut u8,
                screen_id_bytes.len(),
            );
            core::ptr::write_bytes(result_pointer as *mut u8, 0, CLOSE_RESULT_BYTES as usize);
        }
        // SAFETY: the host synchronously copies the ID and writes one boolean byte to the output.
        let status = unsafe {
            imported_close_screen(
                screen_id_pointer as i32,
                screen_id_bytes.len() as i32,
                result_pointer as i32,
                CLOSE_RESULT_BYTES as i32,
            )
        };
        let result = if status != 0 {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected the client screen close request",
            ))
        } else {
            // SAFETY: the host initialized the one-byte result on success.
            match unsafe { core::ptr::read(result_pointer as *const u8) } {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(SjmlError::new(
                    ErrorCode::InvalidTransfer,
                    "host returned an invalid client screen close result",
                )),
            }
        };
        // SAFETY: release both exact live allocations after the synchronous import returns.
        unsafe {
            deallocate_host_input(screen_id_pointer, screen_id_bytes.len() as u32);
            deallocate_host_input(result_pointer, CLOSE_RESULT_BYTES);
        }
        result
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = screen_id;
        Err(SjmlError::new(
            ErrorCode::UnknownErrorCode,
            "interactive screens require the wasm32-unknown-unknown target",
        ))
    }
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

fn update_payload(definition_payload: &[u8]) -> SjmlResult<()> {
    let payload = encode_update_payload(definition_payload)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the envelope remains live while the host synchronously copies and validates it.
        let status = unsafe {
            imported_update_screen(
                payload.as_ptr() as usize as u32 as i32,
                payload.len() as u32 as i32,
            )
        };
        if status == 0 {
            Ok(())
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected the client screen update",
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

fn encode_update_payload(definition_payload: &[u8]) -> SjmlResult<Vec<u8>> {
    let payload_length = u32::try_from(definition_payload.len()).map_err(|_| payload_overflow())?;
    let size = 2usize
        .checked_mul(core::mem::size_of::<u32>())
        .and_then(|header| header.checked_add(definition_payload.len()))
        .ok_or_else(payload_overflow)?;
    if size > MAX_CLIENT_SCREEN_PAYLOAD_BYTES {
        return Err(too_large(format!(
            "encoded client screen update exceeds {MAX_CLIENT_SCREEN_PAYLOAD_BYTES} bytes"
        )));
    }
    let mut payload = Vec::with_capacity(size);
    write_u32(&mut payload, CLIENT_SCREEN_UPDATE_SCHEMA_VERSION);
    write_u32(&mut payload, payload_length);
    payload.extend_from_slice(definition_payload);
    debug_assert_eq!(payload.len(), size);
    Ok(payload)
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

fn encode_list_form_definition(definition: &ClientScreenListFormDefinition) -> SjmlResult<Vec<u8>> {
    let capacity = definition.validate()?;
    let mut payload = encode_definition(&definition.screen)?;
    payload.reserve(capacity.saturating_sub(payload.len()));
    payload[..4].copy_from_slice(&CLIENT_SCREEN_LIST_FORM_SCHEMA_VERSION.to_le_bytes());
    write_u32(&mut payload, definition.fields.len() as u32);
    for field in &definition.fields {
        write_string(&mut payload, &field.id)?;
        write_string(&mut payload, &field.label)?;
        write_string(&mut payload, &field.initial_value)?;
        write_u32(&mut payload, field.max_utf8_bytes);
    }
    write_u32(&mut payload, 1);
    write_string(&mut payload, &definition.list.id)?;
    write_string(&mut payload, &definition.list.label)?;
    write_string(
        &mut payload,
        definition.list.initial_selected_id.as_deref().unwrap_or(""),
    )?;
    write_u32(&mut payload, definition.list.options.len() as u32);
    for option in &definition.list.options {
        write_string(&mut payload, &option.id)?;
        write_string(&mut payload, &option.label)?;
    }
    debug_assert!(payload.len() <= MAX_CLIENT_SCREEN_PAYLOAD_BYTES);
    Ok(payload)
}

fn encode_settings_form_definition(
    definition: &ClientScreenSettingsFormDefinition,
) -> SjmlResult<Vec<u8>> {
    let capacity = definition.validate()?;
    let mut payload = encode_definition(&definition.screen)?;
    payload.reserve(capacity.saturating_sub(payload.len()));
    payload[..4].copy_from_slice(&CLIENT_SCREEN_SETTINGS_FORM_SCHEMA_VERSION.to_le_bytes());
    write_u32(&mut payload, definition.fields.len() as u32);
    for field in &definition.fields {
        write_string(&mut payload, &field.id)?;
        write_string(&mut payload, &field.label)?;
        write_string(&mut payload, &field.initial_value)?;
        write_u32(&mut payload, field.max_utf8_bytes);
    }
    write_u32(&mut payload, u32::from(definition.list.is_some()));
    if let Some(list) = &definition.list {
        write_string(&mut payload, &list.id)?;
        write_string(&mut payload, &list.label)?;
        write_string(
            &mut payload,
            list.initial_selected_id.as_deref().unwrap_or(""),
        )?;
        write_u32(&mut payload, list.options.len() as u32);
        for option in &list.options {
            write_string(&mut payload, &option.id)?;
            write_string(&mut payload, &option.label)?;
        }
    }
    write_u32(&mut payload, definition.checkboxes.len() as u32);
    for checkbox in &definition.checkboxes {
        write_string(&mut payload, &checkbox.id)?;
        write_string(&mut payload, &checkbox.label)?;
        write_u32(&mut payload, u32::from(checkbox.initial_checked));
    }
    debug_assert!(payload.len() <= MAX_CLIENT_SCREEN_PAYLOAD_BYTES);
    Ok(payload)
}

fn encode_slider_form_definition(
    definition: &ClientScreenSliderFormDefinition,
) -> SjmlResult<Vec<u8>> {
    let capacity = definition.validate()?;
    let mut payload = encode_definition(&definition.screen)?;
    payload.reserve(capacity.saturating_sub(payload.len()));
    payload[..4].copy_from_slice(&CLIENT_SCREEN_SLIDER_FORM_SCHEMA_VERSION.to_le_bytes());
    write_u32(&mut payload, definition.fields.len() as u32);
    for field in &definition.fields {
        write_string(&mut payload, &field.id)?;
        write_string(&mut payload, &field.label)?;
        write_string(&mut payload, &field.initial_value)?;
        write_u32(&mut payload, field.max_utf8_bytes);
    }
    write_u32(&mut payload, u32::from(definition.list.is_some()));
    if let Some(list) = &definition.list {
        write_string(&mut payload, &list.id)?;
        write_string(&mut payload, &list.label)?;
        write_string(
            &mut payload,
            list.initial_selected_id.as_deref().unwrap_or(""),
        )?;
        write_u32(&mut payload, list.options.len() as u32);
        for option in &list.options {
            write_string(&mut payload, &option.id)?;
            write_string(&mut payload, &option.label)?;
        }
    }
    write_u32(&mut payload, definition.checkboxes.len() as u32);
    for checkbox in &definition.checkboxes {
        write_string(&mut payload, &checkbox.id)?;
        write_string(&mut payload, &checkbox.label)?;
        write_u32(&mut payload, u32::from(checkbox.initial_checked));
    }
    write_string(&mut payload, &definition.slider.id)?;
    write_string(&mut payload, &definition.slider.label)?;
    payload.extend_from_slice(&definition.slider.minimum.to_le_bytes());
    payload.extend_from_slice(&definition.slider.maximum.to_le_bytes());
    payload.extend_from_slice(&definition.slider.step.to_le_bytes());
    payload.extend_from_slice(&definition.slider.initial_value.to_le_bytes());
    debug_assert!(payload.len() <= MAX_CLIENT_SCREEN_PAYLOAD_BYTES);
    Ok(payload)
}

fn encode_multi_select_form_definition(
    definition: &ClientScreenMultiSelectFormDefinition,
) -> SjmlResult<Vec<u8>> {
    let capacity = definition.validate()?;
    let mut payload = encode_definition(&definition.screen)?;
    payload.reserve(capacity.saturating_sub(payload.len()));
    payload[..4].copy_from_slice(&CLIENT_SCREEN_MULTI_SELECT_FORM_SCHEMA_VERSION.to_le_bytes());
    write_u32(&mut payload, definition.fields.len() as u32);
    for field in &definition.fields {
        write_string(&mut payload, &field.id)?;
        write_string(&mut payload, &field.label)?;
        write_string(&mut payload, &field.initial_value)?;
        write_u32(&mut payload, field.max_utf8_bytes);
    }
    write_u32(&mut payload, 1);
    write_string(&mut payload, &definition.list.id)?;
    write_string(&mut payload, &definition.list.label)?;
    write_u32(
        &mut payload,
        definition.list.initial_selected_ids.len() as u32,
    );
    for selected_id in &definition.list.initial_selected_ids {
        write_string(&mut payload, selected_id)?;
    }
    write_u32(&mut payload, definition.list.options.len() as u32);
    for option in &definition.list.options {
        write_string(&mut payload, &option.id)?;
        write_string(&mut payload, &option.label)?;
    }
    debug_assert!(payload.len() <= MAX_CLIENT_SCREEN_PAYLOAD_BYTES);
    Ok(payload)
}

impl ClientScreenDefinitionSource for ClientScreenDefinition {
    fn encode_client_screen_definition(&self) -> SjmlResult<Vec<u8>> {
        encode_definition(self)
    }
}

impl ClientScreenDefinitionSource for ClientScreenFormDefinition {
    fn encode_client_screen_definition(&self) -> SjmlResult<Vec<u8>> {
        encode_form_definition(self)
    }
}

impl ClientScreenDefinitionSource for ClientScreenListFormDefinition {
    fn encode_client_screen_definition(&self) -> SjmlResult<Vec<u8>> {
        encode_list_form_definition(self)
    }
}

impl ClientScreenDefinitionSource for ClientScreenSettingsFormDefinition {
    fn encode_client_screen_definition(&self) -> SjmlResult<Vec<u8>> {
        encode_settings_form_definition(self)
    }
}

impl ClientScreenDefinitionSource for ClientScreenSliderFormDefinition {
    fn encode_client_screen_definition(&self) -> SjmlResult<Vec<u8>> {
        encode_slider_form_definition(self)
    }
}

impl ClientScreenDefinitionSource for ClientScreenMultiSelectFormDefinition {
    fn encode_client_screen_definition(&self) -> SjmlResult<Vec<u8>> {
        encode_multi_select_form_definition(self)
    }
}

fn encode_progress_bars_definition(
    base_definition: &[u8],
    progress_bars: &[ClientScreenProgressBar],
) -> SjmlResult<Vec<u8>> {
    if base_definition.len() < core::mem::size_of::<u32>() {
        return Err(invalid("base client screen definition is truncated"));
    }
    let base_schema = u32::from_le_bytes(
        base_definition[..4]
            .try_into()
            .expect("the base schema has four bytes"),
    );
    if !(CLIENT_SCREEN_SCHEMA_VERSION..=CLIENT_SCREEN_MULTI_SELECT_FORM_SCHEMA_VERSION)
        .contains(&base_schema)
    {
        return Err(invalid(
            "base client screen definition has an unsupported schema",
        ));
    }
    if !(1..=MAX_CLIENT_SCREEN_PROGRESS_BARS).contains(&progress_bars.len()) {
        return Err(invalid(format!(
            "a progress screen must contain between 1 and {MAX_CLIENT_SCREEN_PROGRESS_BARS} bars"
        )));
    }
    let base_length = u32::try_from(base_definition.len()).map_err(|_| payload_overflow())?;
    let mut size = 3usize
        .checked_mul(core::mem::size_of::<u32>())
        .and_then(|size| size.checked_add(base_definition.len()))
        .ok_or_else(payload_overflow)?;
    for bar in progress_bars {
        bar.validate()?;
        size = checked_string_size(size, &bar.label)?;
        size = size
            .checked_add(3 * core::mem::size_of::<u32>())
            .ok_or_else(payload_overflow)?;
    }
    if size > MAX_CLIENT_SCREEN_PAYLOAD_BYTES - 2 * core::mem::size_of::<u32>() {
        return Err(too_large(
            "encoded progress screen leaves no room for the schema-v7 update envelope",
        ));
    }
    let mut payload = Vec::with_capacity(size);
    write_u32(&mut payload, CLIENT_SCREEN_PROGRESS_BAR_SCHEMA_VERSION);
    write_u32(&mut payload, base_length);
    payload.extend_from_slice(base_definition);
    write_u32(&mut payload, progress_bars.len() as u32);
    for bar in progress_bars {
        write_string(&mut payload, &bar.label)?;
        write_u32(&mut payload, bar.minimum as u32);
        write_u32(&mut payload, bar.maximum as u32);
        write_u32(&mut payload, bar.value as u32);
    }
    debug_assert_eq!(payload.len(), size);
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
        close, encode_definition, encode_form_definition, encode_list_form_definition,
        encode_multi_select_form_definition, encode_settings_form_definition,
        encode_slider_form_definition, encode_update_payload, ClientScreenButton,
        ClientScreenCheckbox, ClientScreenDefinition, ClientScreenFormDefinition,
        ClientScreenListFormDefinition, ClientScreenMultiSelectFormDefinition,
        ClientScreenMultiSelectionList, ClientScreenNumericSlider, ClientScreenProgressBar,
        ClientScreenSelectionList, ClientScreenSelectionOption, ClientScreenSettingsFormDefinition,
        ClientScreenSliderFormDefinition, ClientScreenTextField, ClientScreenWithProgressBars,
        MAX_CLIENT_SCREEN_CHECKBOXES, MAX_CLIENT_SCREEN_ID_BYTES,
        MAX_CLIENT_SCREEN_MULTI_SELECT_OPTIONS, MAX_CLIENT_SCREEN_PAYLOAD_BYTES,
        MAX_CLIENT_SCREEN_SLIDER_ABS_VALUE,
    };
    use crate::ErrorCode;

    #[test]
    fn close_validates_screen_ids_before_host_access() {
        assert_eq!(close("").unwrap_err().code(), ErrorCode::InvalidArgument);
        assert_eq!(
            close("Demo:Settings").unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            close(&"a".repeat(MAX_CLIENT_SCREEN_ID_BYTES + 1))
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        #[cfg(not(target_arch = "wasm32"))]
        assert_eq!(
            close("demo:settings").unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
    }

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
    fn encodes_v7_update_envelope_around_a_complete_form_without_exceeding_payload_limit() {
        let screen = ClientScreenDefinition::new(
            "demo:live",
            "Updated title",
            vec!["Changed body".to_owned()],
            vec![ClientScreenButton::new("apply", "Apply").unwrap()],
        )
        .unwrap();
        let form = ClientScreenFormDefinition::new(
            screen,
            vec![ClientScreenTextField::new("name", "Name", "Ada", 32).unwrap()],
        )
        .unwrap();
        let definition = encode_form_definition(&form).unwrap();
        let update = encode_update_payload(&definition).unwrap();

        assert_eq!(u32::from_le_bytes(update[0..4].try_into().unwrap()), 7);
        assert_eq!(
            u32::from_le_bytes(update[4..8].try_into().unwrap()) as usize,
            definition.len()
        );
        assert_eq!(&update[8..], definition.as_slice());
        assert_eq!(
            u32::from_le_bytes(update[8..12].try_into().unwrap()),
            super::CLIENT_SCREEN_FORM_SCHEMA_VERSION
        );
        assert!(update.len() <= MAX_CLIENT_SCREEN_PAYLOAD_BYTES);
        assert_eq!(
            encode_update_payload(&vec![0; MAX_CLIENT_SCREEN_PAYLOAD_BYTES])
                .unwrap_err()
                .code(),
            ErrorCode::PayloadTooLarge
        );
    }

    #[test]
    fn wraps_a_form_in_schema_v8_and_keeps_it_inside_the_v7_update_envelope() {
        let screen = ClientScreenDefinition::new(
            "demo:progress",
            "Downloading",
            vec!["Preparing files".to_owned()],
            vec![ClientScreenButton::new("cancel", "Cancel").unwrap()],
        )
        .unwrap();
        let form = ClientScreenFormDefinition::new(
            screen,
            vec![ClientScreenTextField::new("status", "Status", "Active", 32).unwrap()],
        )
        .unwrap();
        let decorated = ClientScreenWithProgressBars::new(
            form,
            vec![ClientScreenProgressBar::new("Download", 0, 100, 25).unwrap()],
        )
        .unwrap();
        let definition = decorated.encode_definition().unwrap();
        let update = encode_update_payload(&definition).unwrap();

        assert_eq!(
            u32::from_le_bytes(definition[0..4].try_into().unwrap()),
            super::CLIENT_SCREEN_PROGRESS_BAR_SCHEMA_VERSION
        );
        let base_length = u32::from_le_bytes(definition[4..8].try_into().unwrap()) as usize;
        assert_eq!(
            u32::from_le_bytes(definition[8..12].try_into().unwrap()),
            super::CLIENT_SCREEN_FORM_SCHEMA_VERSION
        );
        assert_eq!(
            u32::from_le_bytes(update[0..4].try_into().unwrap()),
            super::CLIENT_SCREEN_UPDATE_SCHEMA_VERSION
        );
        assert_eq!(
            u32::from_le_bytes(update[8..12].try_into().unwrap()),
            super::CLIENT_SCREEN_PROGRESS_BAR_SCHEMA_VERSION
        );
        assert_eq!(
            u32::from_le_bytes(update[12..16].try_into().unwrap()) as usize,
            base_length
        );
        assert_eq!(decorated.progress_bars[0].fraction(), 0.25);
        assert!(update.len() <= MAX_CLIENT_SCREEN_PAYLOAD_BYTES);
    }

    #[test]
    fn progress_bars_reject_empty_or_out_of_range_definitions() {
        let screen = ClientScreenDefinition::new(
            "demo:progress",
            "Progress",
            Vec::new(),
            vec![ClientScreenButton::new("close", "Close").unwrap()],
        )
        .unwrap();
        assert_eq!(
            ClientScreenWithProgressBars::new(screen.clone(), Vec::new())
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            ClientScreenProgressBar::new("Work", 10, 10, 10)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            ClientScreenProgressBar::new("Work", 0, 10, 11)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            ClientScreenProgressBar::new("Work", -1_000_001, 10, 0)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        let bar = ClientScreenProgressBar {
            label: "Work".to_owned(),
            minimum: 0,
            maximum: 10,
            value: 12,
        };
        assert_eq!(
            bar.fraction(),
            1.0,
            "rendering clamps defensively to a full bar"
        );
        assert_eq!(
            ClientScreenWithProgressBars::new(screen, vec![bar])
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
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

    #[test]
    fn encodes_schema_v3_list_form_after_the_v1_prefix_and_v2_fields() {
        let screen = ClientScreenDefinition::new(
            "demo:settings",
            "Settings",
            vec!["Pick a mode".to_owned()],
            vec![ClientScreenButton::new("apply", "Apply").unwrap()],
        )
        .unwrap();
        let form = ClientScreenListFormDefinition::new(
            screen.clone(),
            vec![ClientScreenTextField::new("name", "Name", "Ada", 64).unwrap()],
            ClientScreenSelectionList::new(
                "mode",
                "Mode",
                vec![
                    ClientScreenSelectionOption::new("easy", "Easy").unwrap(),
                    ClientScreenSelectionOption::new("hard", "Hard").unwrap(),
                ],
                Some("hard".to_owned()),
            )
            .unwrap(),
        )
        .unwrap();
        let encoded = encode_list_form_definition(&form).unwrap();
        let mut expected = encode_definition(&screen).unwrap();
        expected[..4].copy_from_slice(&3_u32.to_le_bytes());
        expected.extend_from_slice(&1_u32.to_le_bytes());
        append_string(&mut expected, "name");
        append_string(&mut expected, "Name");
        append_string(&mut expected, "Ada");
        expected.extend_from_slice(&64_u32.to_le_bytes());
        expected.extend_from_slice(&1_u32.to_le_bytes());
        append_string(&mut expected, "mode");
        append_string(&mut expected, "Mode");
        append_string(&mut expected, "hard");
        expected.extend_from_slice(&2_u32.to_le_bytes());
        append_string(&mut expected, "easy");
        append_string(&mut expected, "Easy");
        append_string(&mut expected, "hard");
        append_string(&mut expected, "Hard");
        assert_eq!(encoded, expected);
        assert!(encoded.len() <= MAX_CLIENT_SCREEN_PAYLOAD_BYTES);
    }

    #[test]
    fn permits_empty_text_fields_and_unselected_list_but_enforces_references_and_unique_ids() {
        let screen = ClientScreenDefinition::new(
            "demo:settings",
            "Settings",
            Vec::new(),
            vec![ClientScreenButton::new("apply", "Apply").unwrap()],
        )
        .unwrap();
        let list = || {
            ClientScreenSelectionList::new(
                "mode",
                "Mode",
                vec![
                    ClientScreenSelectionOption::new("easy", "Easy").unwrap(),
                    ClientScreenSelectionOption::new("hard", "Hard").unwrap(),
                ],
                None,
            )
            .unwrap()
        };
        assert!(ClientScreenListFormDefinition::new(screen.clone(), Vec::new(), list()).is_ok());
        assert!(ClientScreenSelectionList::new(
            "mode",
            "Mode",
            vec![
                ClientScreenSelectionOption::new("easy", "Easy").unwrap(),
                ClientScreenSelectionOption::new("easy", "Duplicate").unwrap(),
            ],
            None,
        )
        .is_err());
        assert!(ClientScreenSelectionList::new(
            "mode",
            "Mode",
            vec![ClientScreenSelectionOption::new("easy", "Easy").unwrap()],
            Some("missing".to_owned()),
        )
        .is_err());
        assert!(ClientScreenListFormDefinition::new(
            screen.clone(),
            vec![ClientScreenTextField::new("mode", "Name", "", 32).unwrap()],
            list(),
        )
        .is_err());
        assert!(ClientScreenListFormDefinition::new(
            screen,
            vec![ClientScreenTextField::new("apply", "Name", "", 32).unwrap()],
            list(),
        )
        .is_err());
    }

    #[test]
    fn encodes_schema_v4_settings_form_with_optional_list_and_checkbox_values() {
        let screen = ClientScreenDefinition::new(
            "demo:settings",
            "Settings",
            vec!["Configure options".to_owned()],
            vec![ClientScreenButton::new("save", "Save").unwrap()],
        )
        .unwrap();
        let list = ClientScreenSelectionList::new(
            "mode",
            "Mode",
            vec![
                ClientScreenSelectionOption::new("easy", "Easy").unwrap(),
                ClientScreenSelectionOption::new("hard", "Hard").unwrap(),
            ],
            Some("hard".to_owned()),
        )
        .unwrap();
        let form = ClientScreenSettingsFormDefinition::new(
            screen.clone(),
            vec![ClientScreenTextField::new("name", "Name", "Ada", 64).unwrap()],
            Some(list),
            vec![
                ClientScreenCheckbox::new("sound", "Enable sound", true).unwrap(),
                ClientScreenCheckbox::new("music", "Enable music", false).unwrap(),
                ClientScreenCheckbox::new("cache", "Cache responses", true).unwrap(),
                ClientScreenCheckbox::new("telemetry", "Share usage data", false).unwrap(),
                ClientScreenCheckbox::new("cloud_sync", "Sync settings", true).unwrap(),
                ClientScreenCheckbox::new("compact_ui", "Use compact layout", false).unwrap(),
                ClientScreenCheckbox::new("updates", "Check for updates", true).unwrap(),
                ClientScreenCheckbox::new("analytics", "Send diagnostics", false).unwrap(),
            ],
        )
        .unwrap();
        let encoded = encode_settings_form_definition(&form).unwrap();

        let mut expected = encode_definition(&screen).unwrap();
        expected[..4].copy_from_slice(&4_u32.to_le_bytes());
        expected.extend_from_slice(&1_u32.to_le_bytes());
        append_string(&mut expected, "name");
        append_string(&mut expected, "Name");
        append_string(&mut expected, "Ada");
        expected.extend_from_slice(&64_u32.to_le_bytes());
        expected.extend_from_slice(&1_u32.to_le_bytes());
        append_string(&mut expected, "mode");
        append_string(&mut expected, "Mode");
        append_string(&mut expected, "hard");
        expected.extend_from_slice(&2_u32.to_le_bytes());
        append_string(&mut expected, "easy");
        append_string(&mut expected, "Easy");
        append_string(&mut expected, "hard");
        append_string(&mut expected, "Hard");
        expected.extend_from_slice(&(form.checkboxes.len() as u32).to_le_bytes());
        for checkbox in &form.checkboxes {
            append_string(&mut expected, &checkbox.id);
            append_string(&mut expected, &checkbox.label);
            expected.extend_from_slice(&u32::from(checkbox.initial_checked).to_le_bytes());
        }
        assert_eq!(encoded, expected);
        assert!(encoded.len() <= MAX_CLIENT_SCREEN_PAYLOAD_BYTES);

        let checkbox_only = ClientScreenSettingsFormDefinition::new(
            screen,
            Vec::new(),
            None,
            vec![ClientScreenCheckbox::new("enabled", "Enabled", false).unwrap()],
        )
        .unwrap();
        let checkbox_only_payload = encode_settings_form_definition(&checkbox_only).unwrap();
        assert_eq!(
            u32::from_le_bytes(checkbox_only_payload[..4].try_into().unwrap()),
            4
        );
        assert_eq!(
            u32::from_le_bytes(
                checkbox_only_payload[checkbox_only_payload.len() - 4..]
                    .try_into()
                    .unwrap()
            ),
            0,
            "a checkbox-only definition writes false as the final boolean word"
        );
    }

    #[test]
    fn settings_form_bounds_checkbox_count_labels_and_combined_ids() {
        let screen = || {
            ClientScreenDefinition::new(
                "demo:settings",
                "Settings",
                Vec::new(),
                vec![ClientScreenButton::new("save", "Save").unwrap()],
            )
            .unwrap()
        };
        assert!(
            ClientScreenSettingsFormDefinition::new(screen(), Vec::new(), None, Vec::new())
                .is_err()
        );
        assert!(ClientScreenCheckbox::new("bad?", "Enabled", true).is_err());
        assert!(ClientScreenCheckbox::new("enabled", "", true).is_err());
        let duplicate = ClientScreenCheckbox::new("enabled", "Enabled", true).unwrap();
        assert!(ClientScreenSettingsFormDefinition::new(
            screen(),
            vec![ClientScreenTextField::new("enabled", "Name", "", 32).unwrap()],
            None,
            vec![duplicate.clone()],
        )
        .is_err());
        assert!(ClientScreenSettingsFormDefinition::new(
            screen(),
            Vec::new(),
            None,
            vec![
                duplicate.clone(),
                duplicate.clone(),
                duplicate.clone(),
                duplicate.clone(),
                duplicate
            ],
        )
        .is_err());
        let too_many = (0..=MAX_CLIENT_SCREEN_CHECKBOXES)
            .map(|index| {
                ClientScreenCheckbox::new(format!("checkbox_{index}"), "Checkbox", false).unwrap()
            })
            .collect();
        assert!(
            ClientScreenSettingsFormDefinition::new(screen(), Vec::new(), None, too_many).is_err()
        );
    }

    #[test]
    fn encodes_schema_v5_slider_after_compatible_v4_form_fields() {
        let screen = ClientScreenDefinition::new(
            "demo:volume",
            "Volume",
            vec!["Set a level".to_owned()],
            vec![ClientScreenButton::new("save", "Save").unwrap()],
        )
        .unwrap();
        let slider = ClientScreenNumericSlider::new("level", "Level", -10, 10, 5, 0).unwrap();
        let definition = ClientScreenSliderFormDefinition::new(
            screen.clone(),
            Vec::new(),
            None,
            Vec::new(),
            slider,
        )
        .unwrap();
        let encoded = encode_slider_form_definition(&definition).unwrap();
        let mut expected = encode_definition(&screen).unwrap();
        expected[..4].copy_from_slice(&5_u32.to_le_bytes());
        expected.extend_from_slice(&0_u32.to_le_bytes()); // text fields
        expected.extend_from_slice(&0_u32.to_le_bytes()); // optional list
        expected.extend_from_slice(&0_u32.to_le_bytes()); // checkboxes
        append_string(&mut expected, "level");
        append_string(&mut expected, "Level");
        for value in [-10_i32, 10, 5, 0] {
            expected.extend_from_slice(&value.to_le_bytes());
        }
        assert_eq!(encoded, expected);
        assert!(encoded.len() <= MAX_CLIENT_SCREEN_PAYLOAD_BYTES);
    }

    #[test]
    fn slider_validation_checks_step_alignment_range_and_element_ids() {
        assert!(ClientScreenNumericSlider::new("level", "Level", 0, 10, 3, 3).is_err());
        assert!(ClientScreenNumericSlider::new("level", "Level", 0, 10, 2, 3).is_err());
        assert!(ClientScreenNumericSlider::new("level", "Level", 0, 10, 0, 0).is_err());
        assert!(ClientScreenNumericSlider::new("level", "Level", 0, 10, 1, 11).is_err());
        assert!(ClientScreenNumericSlider::new("level", "", 0, 10, 1, 0).is_err());
        assert!(ClientScreenNumericSlider::new("level", "Level", 0, 0, 1, 0).is_err());
        assert!(ClientScreenNumericSlider::new(
            "level",
            "Level",
            -MAX_CLIENT_SCREEN_SLIDER_ABS_VALUE - 1,
            10,
            1,
            0
        )
        .is_err());
        assert_eq!(
            ClientScreenNumericSlider::new(
                "level",
                "Level",
                -MAX_CLIENT_SCREEN_SLIDER_ABS_VALUE,
                MAX_CLIENT_SCREEN_SLIDER_ABS_VALUE,
                2 * MAX_CLIENT_SCREEN_SLIDER_ABS_VALUE,
                -MAX_CLIENT_SCREEN_SLIDER_ABS_VALUE,
            )
            .unwrap()
            .initial_value,
            -MAX_CLIENT_SCREEN_SLIDER_ABS_VALUE
        );

        let screen = ClientScreenDefinition::new(
            "demo:volume",
            "Volume",
            Vec::new(),
            vec![ClientScreenButton::new("save", "Save").unwrap()],
        )
        .unwrap();
        let slider = ClientScreenNumericSlider::new("save", "Level", 0, 10, 1, 0).unwrap();
        assert!(ClientScreenSliderFormDefinition::new(
            screen,
            Vec::new(),
            None,
            Vec::new(),
            slider,
        )
        .is_err());
    }

    #[test]
    fn encodes_schema_v6_multi_select_form_in_its_defined_wire_order() {
        let screen = ClientScreenDefinition::new(
            "demo:features",
            "Features",
            vec!["Choose any options".to_owned()],
            vec![ClientScreenButton::new("save", "Save").unwrap()],
        )
        .unwrap();
        let form = ClientScreenMultiSelectFormDefinition::new(
            screen.clone(),
            vec![ClientScreenTextField::new("name", "Name", "Ada", 64).unwrap()],
            ClientScreenMultiSelectionList::new(
                "features",
                "Enabled features",
                vec![
                    ClientScreenSelectionOption::new("chat", "Chat").unwrap(),
                    ClientScreenSelectionOption::new("homes", "Homes").unwrap(),
                ],
                vec!["homes".to_owned()],
            )
            .unwrap(),
        )
        .unwrap();
        let encoded = encode_multi_select_form_definition(&form).unwrap();
        let mut expected = encode_definition(&screen).unwrap();
        expected[..4].copy_from_slice(&6_u32.to_le_bytes());
        expected.extend_from_slice(&1_u32.to_le_bytes());
        append_string(&mut expected, "name");
        append_string(&mut expected, "Name");
        append_string(&mut expected, "Ada");
        expected.extend_from_slice(&64_u32.to_le_bytes());
        expected.extend_from_slice(&1_u32.to_le_bytes());
        append_string(&mut expected, "features");
        append_string(&mut expected, "Enabled features");
        expected.extend_from_slice(&1_u32.to_le_bytes());
        append_string(&mut expected, "homes");
        expected.extend_from_slice(&2_u32.to_le_bytes());
        append_string(&mut expected, "chat");
        append_string(&mut expected, "Chat");
        append_string(&mut expected, "homes");
        append_string(&mut expected, "Homes");
        assert_eq!(encoded, expected);
        assert!(encoded.len() <= MAX_CLIENT_SCREEN_PAYLOAD_BYTES);
    }

    #[test]
    fn multi_select_form_validates_option_bounds_initial_ids_and_collisions() {
        let screen = || {
            ClientScreenDefinition::new(
                "demo:features",
                "Features",
                Vec::new(),
                vec![ClientScreenButton::new("save", "Save").unwrap()],
            )
            .unwrap()
        };
        let options = || {
            vec![
                ClientScreenSelectionOption::new("chat", "Chat").unwrap(),
                ClientScreenSelectionOption::new("homes", "Homes").unwrap(),
            ]
        };
        assert!(ClientScreenMultiSelectionList::new(
            "features",
            "Features",
            options(),
            vec!["unknown".to_owned()]
        )
        .is_err());
        assert!(ClientScreenMultiSelectionList::new(
            "features",
            "Features",
            options(),
            vec!["chat".to_owned(), "chat".to_owned()]
        )
        .is_err());
        assert!(ClientScreenMultiSelectionList::new(
            "features",
            "Features",
            (0..=MAX_CLIENT_SCREEN_MULTI_SELECT_OPTIONS)
                .map(
                    |index| ClientScreenSelectionOption::new(format!("option_{index}"), "Option")
                        .unwrap()
                )
                .collect(),
            Vec::new(),
        )
        .is_err());
        assert!(ClientScreenMultiSelectFormDefinition::new(
            screen(),
            vec![ClientScreenTextField::new("features", "Name", "", 32).unwrap()],
            ClientScreenMultiSelectionList::new("features", "Features", options(), Vec::new())
                .unwrap(),
        )
        .is_err());
        let maximum = (0..MAX_CLIENT_SCREEN_MULTI_SELECT_OPTIONS)
            .map(|index| {
                ClientScreenSelectionOption::new(format!("option_{index}"), "Option").unwrap()
            })
            .collect::<Vec<_>>();
        assert!(ClientScreenMultiSelectionList::new(
            "features",
            "Features",
            maximum.clone(),
            maximum.iter().map(|option| option.id.clone()).collect(),
        )
        .is_ok());
    }

    fn append_string(target: &mut Vec<u8>, value: &str) {
        target.extend_from_slice(&(value.len() as u32).to_le_bytes());
        target.extend_from_slice(value.as_bytes());
    }
}
