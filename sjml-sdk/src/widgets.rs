//! Small high-level GUI controls built from the bounded render and screen-input APIs.
//!
//! Widgets are guest-owned values: call [`Button::draw`] or [`TextField::draw`] from a
//! `RenderLayer::Screen` callback, and pass each `ScreenInputEvent` to the corresponding
//! `handle_input` method from `screen_input::subscribe`.
//!
//! ```
//! use sjml::{Button, ScreenInputEvent, TextField, WidgetRect};
//!
//! let button = Button::new(WidgetRect::new(20.0, 20.0, 100.0, 20.0).unwrap(), "Apply").unwrap();
//! let clicked = button.handle_input(ScreenInputEvent::MouseButtonPressed {
//!     mouse_x: 24.0, mouse_y: 24.0, button: 0, modifiers: 0,
//! });
//! assert!(clicked);
//!
//! let mut field = TextField::new(WidgetRect::new(20.0, 48.0, 180.0, 20.0).unwrap(), 256).unwrap();
//! field.handle_input(ScreenInputEvent::MouseButtonPressed {
//!     mouse_x: 24.0, mouse_y: 52.0, button: 0, modifiers: 0,
//! });
//! for code_unit in "SJML".encode_utf16() {
//!     field.handle_input(ScreenInputEvent::CharacterTyped { code_unit, modifiers: 0 });
//! }
//! assert_eq!(field.value(), "SJML");
//! ```
//!
//! Draw the status bar and pointer tooltip from a screen render callback:
//!
//! ```
//! use sjml::prelude::*;
//!
//! #[allow(dead_code)]
//! fn install_status_widgets() -> SjmlResult<()> {
//!     let tooltip = Tooltip::new(
//!         WidgetRect::new(20.0, 20.0, 100.0, 20.0)?,
//!         "Additional status details",
//!     )?;
//!     let mut progress = ProgressBar::new(
//!         WidgetRect::new(20.0, 52.0, 160.0, 8.0)?,
//!         0.65,
//!     )?;
//!     progress.set_fraction(0.75)?;
//!
//!     let _screen = render::subscribe(RenderLayer::Screen, move |context| {
//!         progress.draw(context)?;
//!         let _tooltip_visible = tooltip.draw(context)?;
//!         Ok(())
//!     })?;
//!     Ok(())
//! }
//! ```

use crate::render::{RenderContext, MAX_RENDER_TEXT_BYTES, MAX_TOOLTIP_BYTES};
use crate::screen_input::ScreenInputEvent;
use crate::{ErrorCode, SjmlError, SjmlResult};

/// Maximum UTF-8 storage per [`TextField`].
pub const MAX_TEXT_FIELD_BYTES: usize = 4096;
const MAX_WIDGET_COORDINATE: f32 = 1_048_576.0;
const FONT_LINE_HEIGHT: f32 = 9.0;
const GLFW_KEY_ESCAPE: i32 = 256;
const GLFW_KEY_ENTER: i32 = 257;
const GLFW_KEY_BACKSPACE: i32 = 259;
const GLFW_KEY_DELETE: i32 = 261;
const GLFW_KEY_RIGHT: i32 = 262;
const GLFW_KEY_LEFT: i32 = 263;
const GLFW_KEY_END: i32 = 269;
const GLFW_KEY_HOME: i32 = 268;
const GLFW_KEY_KP_ENTER: i32 = 335;
const MODIFIER_CONTROL_ALT_SUPER: u8 = 0x0e;

/// Logical GUI rectangle shared by the built-in controls.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WidgetRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl WidgetRect {
    /// Creates a finite, non-negative rectangle whose far edge stays inside the host GUI bound.
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> SjmlResult<Self> {
        let bounds = Self {
            x,
            y,
            width,
            height,
        };
        validate_widget_rect(bounds)?;
        Ok(bounds)
    }

    /// Returns whether the logical pointer is inside the half-open rectangle.
    pub fn contains(self, x: f32, y: f32) -> bool {
        x.is_finite()
            && y.is_finite()
            && x >= self.x
            && y >= self.y
            && x < self.x + self.width
            && y < self.y + self.height
    }
}

fn validate_widget_rect(bounds: WidgetRect) -> SjmlResult<()> {
    if [bounds.x, bounds.y, bounds.width, bounds.height]
        .into_iter()
        .any(|value| !value.is_finite() || value.abs() > MAX_WIDGET_COORDINATE)
        || bounds.width < 0.0
        || bounds.height < 0.0
        || (bounds.x + bounds.width).abs() > MAX_WIDGET_COORDINATE
        || (bounds.y + bounds.height).abs() > MAX_WIDGET_COORDINATE
    {
        return Err(invalid("widget bounds are outside the finite GUI range"));
    }
    Ok(())
}

fn validate_visible_bounds(bounds: WidgetRect, name: &'static str) -> SjmlResult<()> {
    validate_widget_rect(bounds)?;
    if bounds.width <= 0.0 || bounds.height <= 0.0 {
        return Err(invalid(name));
    }
    Ok(())
}

/// A guest-owned button with bounded pointer handling and a ready-to-draw default style.
#[derive(Clone, Debug)]
pub struct Button {
    bounds: WidgetRect,
    label: String,
    normal_argb: u32,
    hovered_argb: u32,
    text_argb: u32,
}

impl Button {
    /// Creates a button. It activates on a left-button press inside its rectangle.
    pub fn new(bounds: WidgetRect, label: impl Into<String>) -> SjmlResult<Self> {
        let label = label.into();
        validate_label(&label)?;
        Ok(Self {
            bounds,
            label,
            normal_argb: 0xff40_4650,
            hovered_argb: 0xff5c_6878,
            text_argb: 0xffff_ffff,
        })
    }

    /// Sets the normal, hover, and label colors in ARGB order.
    pub const fn colors(mut self, normal_argb: u32, hovered_argb: u32, text_argb: u32) -> Self {
        self.normal_argb = normal_argb;
        self.hovered_argb = hovered_argb;
        self.text_argb = text_argb;
        self
    }

    /// The rectangle occupied by this control.
    pub const fn bounds(&self) -> WidgetRect {
        self.bounds
    }

    /// The label drawn inside the control.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Draws the button using the screen callback's pointer position for its hover state.
    pub fn draw(&self, context: &mut RenderContext) -> SjmlResult<()> {
        let frame = context.frame();
        let hovered = self.bounds.contains(frame.mouse_x, frame.mouse_y);
        let background = if hovered {
            self.hovered_argb
        } else {
            self.normal_argb
        };
        context.draw_rect(
            self.bounds.x,
            self.bounds.y,
            self.bounds.width,
            self.bounds.height,
            background,
        )?;
        let text_x = self.bounds.x
            + ((self.bounds.width - estimated_text_width(&self.label)) / 2.0).max(4.0);
        let text_y = self.bounds.y + ((self.bounds.height - FONT_LINE_HEIGHT) / 2.0).max(0.0);
        context.draw_text(&self.label, text_x, text_y, self.text_argb, true)
    }

    /// Returns true for a left-button press inside the button.
    pub fn handle_input(&self, event: ScreenInputEvent) -> bool {
        matches!(
            event,
            ScreenInputEvent::MouseButtonPressed {
                mouse_x,
                mouse_y,
                button: 0,
                ..
            } if self.bounds.contains(mouse_x, mouse_y)
        )
    }
}

/// A bounded hover target that submits a standard Minecraft tooltip while the pointer is over it.
#[derive(Clone, Debug)]
pub struct Tooltip {
    bounds: WidgetRect,
    text: String,
}

impl Tooltip {
    /// Creates a tooltip for a non-empty hover region. Text is limited to 4,096 UTF-8 bytes.
    pub fn new(bounds: WidgetRect, text: impl Into<String>) -> SjmlResult<Self> {
        validate_visible_bounds(bounds, "tooltip bounds must have positive width and height")?;
        let text = text.into();
        if text.is_empty() {
            return Err(invalid("tooltip text must not be empty"));
        }
        if text.len() > MAX_TOOLTIP_BYTES {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "tooltip text exceeds the 4096-byte render limit",
            ));
        }
        Ok(Self { bounds, text })
    }

    /// The rectangle whose hover displays this tooltip.
    pub const fn bounds(&self) -> WidgetRect {
        self.bounds
    }

    /// The tooltip text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Whether the pointer is inside the tooltip's half-open hover rectangle.
    pub fn is_hovered(&self, mouse_x: f32, mouse_y: f32) -> bool {
        self.bounds.contains(mouse_x, mouse_y)
    }

    /// Draws the tooltip at the current pointer position when it is over the hover rectangle.
    /// Returns `true` when a tooltip command was added to the render batch.
    pub fn draw(&self, context: &mut RenderContext) -> SjmlResult<bool> {
        let frame = context.frame();
        if !self.is_hovered(frame.mouse_x, frame.mouse_y) {
            return Ok(false);
        }
        context.draw_tooltip(&self.text, frame.mouse_x, frame.mouse_y)?;
        Ok(true)
    }
}

/// A guest-owned horizontal progress indicator backed by the bounded render progress command.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProgressBar {
    bounds: WidgetRect,
    fraction: f32,
    background_argb: u32,
    foreground_argb: u32,
}

impl ProgressBar {
    /// Creates a progress bar with a finite fill fraction in `[0, 1]`.
    pub fn new(bounds: WidgetRect, fraction: f32) -> SjmlResult<Self> {
        validate_visible_bounds(
            bounds,
            "progress bar bounds must have positive width and height",
        )?;
        validate_progress_fraction(fraction)?;
        Ok(Self {
            bounds,
            fraction,
            background_argb: 0xff30_3030,
            foreground_argb: 0xff40_c060,
        })
    }

    /// Sets the background and filled portions' ARGB colors.
    pub const fn colors(mut self, background_argb: u32, foreground_argb: u32) -> Self {
        self.background_argb = background_argb;
        self.foreground_argb = foreground_argb;
        self
    }

    /// The rectangle occupied by this control.
    pub const fn bounds(&self) -> WidgetRect {
        self.bounds
    }

    /// The current fill fraction.
    pub const fn fraction(&self) -> f32 {
        self.fraction
    }

    /// Replaces the fill fraction, preserving the current value if validation fails.
    pub fn set_fraction(&mut self, fraction: f32) -> SjmlResult<()> {
        validate_progress_fraction(fraction)?;
        self.fraction = fraction;
        Ok(())
    }

    /// Draws the bar through `RenderContext::draw_progress`.
    pub fn draw(&self, context: &mut RenderContext) -> SjmlResult<()> {
        context.draw_progress(
            self.bounds.x,
            self.bounds.y,
            self.bounds.width,
            self.bounds.height,
            self.fraction,
            self.background_argb,
            self.foreground_argb,
        )
    }
}

fn validate_progress_fraction(fraction: f32) -> SjmlResult<()> {
    if !fraction.is_finite() || !(0.0..=1.0).contains(&fraction) {
        return Err(invalid(
            "progress fraction must be finite and within [0, 1]",
        ));
    }
    Ok(())
}

/// Result of handling one event in a text field.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextFieldAction {
    /// The event did not affect this text field and remains available to other screen handling.
    Ignored,
    /// The event changed focus but is left available to Minecraft.
    Blurred,
    /// The field consumed an event without changing its value.
    Consumed,
    /// The field's value changed.
    Changed,
    /// Enter was pressed while the field was focused.
    Submitted,
}

impl TextFieldAction {
    /// Whether a screen-input callback should return `true` for this action.
    pub const fn consumes_input(self) -> bool {
        matches!(self, Self::Consumed | Self::Changed | Self::Submitted)
    }
}

/// A bounded, single-line guest-owned text field.
#[derive(Clone, Debug)]
pub struct TextField {
    bounds: WidgetRect,
    value: String,
    placeholder: String,
    max_bytes: usize,
    cursor: usize,
    focused: bool,
    pending_high_surrogate: Option<u16>,
    background_argb: u32,
    focused_argb: u32,
    text_argb: u32,
    placeholder_argb: u32,
    cursor_argb: u32,
}

impl TextField {
    /// Creates an empty field. `max_bytes` must be between one and 4,096 UTF-8 bytes.
    pub fn new(bounds: WidgetRect, max_bytes: usize) -> SjmlResult<Self> {
        if !(1..=MAX_TEXT_FIELD_BYTES).contains(&max_bytes) {
            return Err(invalid("text field byte limit must be in [1, 4096]"));
        }
        if bounds.width < 12.0 || bounds.height < FONT_LINE_HEIGHT + 2.0 {
            return Err(invalid(
                "text field bounds are too small for one line of text",
            ));
        }
        Ok(Self {
            bounds,
            value: String::new(),
            placeholder: String::new(),
            max_bytes,
            cursor: 0,
            focused: false,
            pending_high_surrogate: None,
            background_argb: 0xff18_1c22,
            focused_argb: 0xff4b_a3ff,
            text_argb: 0xffff_ffff,
            placeholder_argb: 0xff9a_a3ad,
            cursor_argb: 0xffff_ffff,
        })
    }

    /// Sets text shown while the value is empty. The text uses the render API's string bound.
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> SjmlResult<Self> {
        let placeholder = placeholder.into();
        validate_label(&placeholder)?;
        self.placeholder = placeholder;
        Ok(self)
    }

    /// Sets the field background, focus border, value, placeholder, and cursor colors.
    pub const fn colors(
        mut self,
        background_argb: u32,
        focused_argb: u32,
        text_argb: u32,
        placeholder_argb: u32,
        cursor_argb: u32,
    ) -> Self {
        self.background_argb = background_argb;
        self.focused_argb = focused_argb;
        self.text_argb = text_argb;
        self.placeholder_argb = placeholder_argb;
        self.cursor_argb = cursor_argb;
        self
    }

    /// Current field value.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Current logical focus state.
    pub const fn is_focused(&self) -> bool {
        self.focused
    }

    /// Sets the field value and places the cursor at its end.
    pub fn set_value(&mut self, value: &str) -> SjmlResult<()> {
        validate_editable_text(value)?;
        if value.len() > self.max_bytes {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "text field value exceeds its configured UTF-8 byte limit",
            ));
        }
        self.value.clear();
        self.value.push_str(value);
        self.cursor = self.value.len();
        self.pending_high_surrogate = None;
        Ok(())
    }

    /// Draws a bounded background, visible value/placeholder, and an insertion cursor.
    pub fn draw(&self, context: &mut RenderContext) -> SjmlResult<()> {
        let border = if self.focused {
            self.focused_argb
        } else {
            self.background_argb
        };
        context.draw_rect(
            self.bounds.x,
            self.bounds.y,
            self.bounds.width,
            self.bounds.height,
            border,
        )?;
        let inset = if self.focused { 1.0 } else { 0.0 };
        context.draw_rect(
            self.bounds.x + inset,
            self.bounds.y + inset,
            (self.bounds.width - 2.0 * inset).max(0.0),
            (self.bounds.height - 2.0 * inset).max(0.0),
            self.background_argb,
        )?;

        let inner_x = self.bounds.x + 4.0;
        let inner_width = (self.bounds.width - 8.0).max(0.0);
        let line_y = self.bounds.y + ((self.bounds.height - FONT_LINE_HEIGHT) / 2.0).max(0.0);
        let (display, display_argb, cursor_x) = self.visible_content(inner_width);
        if !display.is_empty() {
            context.draw_text(&display, inner_x, line_y, display_argb, false)?;
        }
        if self.focused {
            context.draw_rect(
                inner_x + cursor_x,
                line_y - 1.0,
                1.0,
                FONT_LINE_HEIGHT + 2.0,
                self.cursor_argb,
            )?;
        }
        Ok(())
    }

    /// Applies one screen event. Use `action.consumes_input()` as the callback result when this
    /// field is the active control.
    pub fn handle_input(&mut self, event: ScreenInputEvent) -> TextFieldAction {
        match event {
            ScreenInputEvent::MouseButtonPressed {
                mouse_x,
                mouse_y,
                button: 0,
                ..
            } if self.bounds.contains(mouse_x, mouse_y) => {
                self.focused = true;
                self.pending_high_surrogate = None;
                self.cursor = self.cursor_for_x(mouse_x);
                TextFieldAction::Consumed
            }
            ScreenInputEvent::MouseButtonPressed { .. } if self.focused => {
                self.focused = false;
                self.pending_high_surrogate = None;
                TextFieldAction::Blurred
            }
            ScreenInputEvent::CharacterTyped {
                code_unit,
                modifiers,
            } if self.focused => self.handle_character(code_unit, modifiers),
            ScreenInputEvent::KeyPressed {
                key_code,
                modifiers,
                ..
            } if self.focused => self.handle_key(key_code, modifiers),
            _ => TextFieldAction::Ignored,
        }
    }

    fn handle_character(&mut self, code_unit: u16, modifiers: u8) -> TextFieldAction {
        if modifiers & MODIFIER_CONTROL_ALT_SUPER != 0 {
            self.pending_high_surrogate = None;
            return TextFieldAction::Consumed;
        }
        if (0xd800..=0xdbff).contains(&code_unit) {
            self.pending_high_surrogate = Some(code_unit);
            return TextFieldAction::Consumed;
        }
        let character = if (0xdc00..=0xdfff).contains(&code_unit) {
            let Some(high) = self.pending_high_surrogate.take() else {
                return TextFieldAction::Consumed;
            };
            let high = u32::from(high - 0xd800);
            let low = u32::from(code_unit - 0xdc00);
            char::from_u32(0x1_0000 + ((high << 10) | low))
        } else {
            self.pending_high_surrogate = None;
            char::from_u32(u32::from(code_unit))
        };
        let Some(character) = character.filter(|character| !character.is_control()) else {
            return TextFieldAction::Consumed;
        };
        self.insert_character(character)
    }

    fn handle_key(&mut self, key_code: i32, modifiers: u8) -> TextFieldAction {
        self.pending_high_surrogate = None;
        if key_code == GLFW_KEY_ESCAPE {
            self.focused = false;
            return TextFieldAction::Blurred;
        }
        if key_code == GLFW_KEY_ENTER || key_code == GLFW_KEY_KP_ENTER {
            return TextFieldAction::Submitted;
        }
        if modifiers & MODIFIER_CONTROL_ALT_SUPER != 0 {
            return TextFieldAction::Consumed;
        }
        match key_code {
            GLFW_KEY_BACKSPACE => self.delete_before_cursor(),
            GLFW_KEY_DELETE => self.delete_after_cursor(),
            GLFW_KEY_LEFT => {
                self.cursor = previous_boundary(&self.value, self.cursor);
                TextFieldAction::Consumed
            }
            GLFW_KEY_RIGHT => {
                self.cursor = next_boundary(&self.value, self.cursor);
                TextFieldAction::Consumed
            }
            GLFW_KEY_HOME => {
                self.cursor = 0;
                TextFieldAction::Consumed
            }
            GLFW_KEY_END => {
                self.cursor = self.value.len();
                TextFieldAction::Consumed
            }
            _ => TextFieldAction::Consumed,
        }
    }

    fn insert_character(&mut self, character: char) -> TextFieldAction {
        if self.value.len() + character.len_utf8() > self.max_bytes {
            return TextFieldAction::Consumed;
        }
        self.value.insert(self.cursor, character);
        self.cursor += character.len_utf8();
        TextFieldAction::Changed
    }

    fn delete_before_cursor(&mut self) -> TextFieldAction {
        if self.cursor == 0 {
            return TextFieldAction::Consumed;
        }
        let previous = previous_boundary(&self.value, self.cursor);
        self.value.replace_range(previous..self.cursor, "");
        self.cursor = previous;
        TextFieldAction::Changed
    }

    fn delete_after_cursor(&mut self) -> TextFieldAction {
        if self.cursor >= self.value.len() {
            return TextFieldAction::Consumed;
        }
        let next = next_boundary(&self.value, self.cursor);
        self.value.replace_range(self.cursor..next, "");
        TextFieldAction::Changed
    }

    fn cursor_for_x(&self, mouse_x: f32) -> usize {
        let target = (mouse_x - (self.bounds.x + 4.0)).max(0.0);
        let mut width = 0.0;
        let mut cursor = 0;
        for (index, character) in self.value.char_indices() {
            let glyph_width = estimated_glyph_width(character);
            if width + glyph_width / 2.0 >= target {
                return index;
            }
            width += glyph_width;
            cursor = index + character.len_utf8();
        }
        cursor
    }

    fn visible_content(&self, available_width: f32) -> (String, u32, f32) {
        if self.value.is_empty() {
            return (self.placeholder.clone(), self.placeholder_argb, 0.0);
        }
        let chars: Vec<(usize, char)> = self.value.char_indices().collect();
        let cursor_chars = chars.partition_point(|(byte, _)| *byte < self.cursor);
        let max_width = available_width.max(0.0);
        let mut start = cursor_chars.min(chars.len());
        let mut visible_width = 0.0;
        while start > 0 {
            let glyph_width = estimated_glyph_width(chars[start - 1].1);
            if visible_width + glyph_width > max_width {
                break;
            }
            visible_width += glyph_width;
            start -= 1;
        }
        let mut end = cursor_chars;
        while end < chars.len() {
            let glyph_width = estimated_glyph_width(chars[end].1);
            if visible_width + glyph_width > max_width {
                break;
            }
            visible_width += glyph_width;
            end += 1;
        }
        let begin_byte = chars.get(start).map_or(0, |(byte, _)| *byte);
        let end_byte = chars.get(end).map_or(self.value.len(), |(byte, _)| *byte);
        let text = self.value[begin_byte..end_byte].to_owned();
        let cursor_width: f32 = chars[start..cursor_chars]
            .iter()
            .map(|(_, character)| estimated_glyph_width(*character))
            .sum();
        (text, self.text_argb, cursor_width.min(max_width))
    }
}

fn previous_boundary(value: &str, cursor: usize) -> usize {
    value[..cursor]
        .char_indices()
        .next_back()
        .map_or(0, |(index, _)| index)
}

fn next_boundary(value: &str, cursor: usize) -> usize {
    value[cursor..]
        .char_indices()
        .nth(1)
        .map_or(value.len(), |(offset, _)| cursor + offset)
}

fn estimated_text_width(text: &str) -> f32 {
    text.chars().map(estimated_glyph_width).sum()
}

fn estimated_glyph_width(character: char) -> f32 {
    match character {
        'i' | 'l' | '!' | '.' | ',' | ':' | ';' | '\'' | '|' => 2.0,
        ' ' | 'I' | 't' | 'f' | '(' | ')' | '[' | ']' => 4.0,
        character if character.is_ascii() => 6.0,
        character if character as u32 >= 0x2e80 => 9.0,
        _ => 6.0,
    }
}

fn validate_label(label: &str) -> SjmlResult<()> {
    if label.len() > MAX_RENDER_TEXT_BYTES {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "widget label exceeds the render text byte limit",
        ));
    }
    Ok(())
}

fn validate_editable_text(value: &str) -> SjmlResult<()> {
    validate_label(value)?;
    if value.chars().any(char::is_control) {
        return Err(invalid(
            "text field value cannot contain control characters",
        ));
    }
    Ok(())
}

fn invalid(message: &'static str) -> SjmlError {
    SjmlError::new(ErrorCode::InvalidArgument, message)
}

#[cfg(test)]
mod tests {
    use super::{
        Button, ProgressBar, TextField, TextFieldAction, Tooltip, WidgetRect, GLFW_KEY_BACKSPACE,
        GLFW_KEY_ENTER, GLFW_KEY_LEFT, MAX_TEXT_FIELD_BYTES,
    };
    use crate::render::MAX_TOOLTIP_BYTES;
    use crate::screen_input::ScreenInputEvent;
    use crate::ErrorCode;

    fn rect() -> WidgetRect {
        WidgetRect::new(10.0, 20.0, 120.0, 18.0).unwrap()
    }

    #[test]
    fn button_activates_only_on_left_click_inside_its_rectangle() {
        let button = Button::new(rect(), "Apply").unwrap();
        assert!(button.handle_input(ScreenInputEvent::MouseButtonPressed {
            mouse_x: 12.0,
            mouse_y: 24.0,
            button: 0,
            modifiers: 0,
        }));
        assert!(!button.handle_input(ScreenInputEvent::MouseButtonPressed {
            mouse_x: 12.0,
            mouse_y: 24.0,
            button: 1,
            modifiers: 0,
        }));
        assert!(!button.handle_input(ScreenInputEvent::MouseButtonPressed {
            mouse_x: 140.0,
            mouse_y: 24.0,
            button: 0,
            modifiers: 0,
        }));
    }

    #[test]
    fn tooltip_checks_bounded_text_and_finite_hover_geometry() {
        let tooltip = Tooltip::new(rect(), "More details").unwrap();
        assert_eq!(tooltip.bounds(), rect());
        assert_eq!(tooltip.text(), "More details");
        assert!(tooltip.is_hovered(10.0, 20.0));
        assert!(!tooltip.is_hovered(130.0, 38.0));
        assert!(!tooltip.is_hovered(f32::NAN, 20.0));

        assert_eq!(
            Tooltip::new(rect(), "x".repeat(MAX_TOOLTIP_BYTES + 1))
                .unwrap_err()
                .code(),
            ErrorCode::PayloadTooLarge
        );
        assert_eq!(
            Tooltip::new(rect(), "😀".repeat(MAX_TOOLTIP_BYTES / 4 + 1))
                .unwrap_err()
                .code(),
            ErrorCode::PayloadTooLarge
        );
        assert!(Tooltip::new(rect(), "").is_err());
        assert_eq!(
            Tooltip::new(
                WidgetRect {
                    x: f32::INFINITY,
                    y: 0.0,
                    width: 1.0,
                    height: 1.0,
                },
                "tip",
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert!(Tooltip::new(
            WidgetRect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 1.0,
            },
            "tip",
        )
        .is_err());
    }

    #[test]
    fn progress_bar_checks_fraction_and_rectangle_bounds() {
        let mut bar = ProgressBar::new(rect(), 0.5)
            .unwrap()
            .colors(0xff10_2030, 0xff40_c060);
        assert_eq!(bar.bounds(), rect());
        assert_eq!(bar.fraction(), 0.5);
        bar.set_fraction(1.0).unwrap();
        assert_eq!(bar.fraction(), 1.0);
        bar.set_fraction(0.0).unwrap();
        assert_eq!(bar.fraction(), 0.0);

        for invalid_fraction in [f32::NAN, f32::INFINITY, -0.01, 1.01] {
            assert_eq!(
                bar.set_fraction(invalid_fraction).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
            assert_eq!(bar.fraction(), 0.0, "failed updates preserve the old value");
        }
        assert!(ProgressBar::new(
            WidgetRect {
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: 0.0,
            },
            0.5,
        )
        .is_err());
        assert!(ProgressBar::new(
            WidgetRect {
                x: 1_048_576.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
            0.5,
        )
        .is_err());
        assert_eq!(
            ProgressBar::new(rect(), f32::NAN).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn text_field_focuses_edits_utf16_surrogate_pairs_and_submits() {
        let mut field = TextField::new(rect(), 16).unwrap();
        assert_eq!(
            field.handle_input(ScreenInputEvent::MouseButtonPressed {
                mouse_x: 20.0,
                mouse_y: 25.0,
                button: 0,
                modifiers: 0,
            }),
            TextFieldAction::Consumed
        );
        assert_eq!(
            field.handle_input(ScreenInputEvent::CharacterTyped {
                code_unit: 0xd83d,
                modifiers: 0,
            }),
            TextFieldAction::Consumed
        );
        assert_eq!(field.value(), "");
        assert_eq!(
            field.handle_input(ScreenInputEvent::CharacterTyped {
                code_unit: 0xde00,
                modifiers: 0,
            }),
            TextFieldAction::Changed
        );
        assert_eq!(field.value(), "😀");
        assert_eq!(
            field.handle_input(ScreenInputEvent::KeyPressed {
                key_code: GLFW_KEY_LEFT,
                scan_code: 0,
                modifiers: 0,
                repeat: false,
            }),
            TextFieldAction::Consumed
        );
        assert_eq!(
            field.handle_input(ScreenInputEvent::CharacterTyped {
                code_unit: b'x' as u16,
                modifiers: 0,
            }),
            TextFieldAction::Changed
        );
        assert_eq!(field.value(), "x😀");
        assert_eq!(
            field.handle_input(ScreenInputEvent::KeyPressed {
                key_code: GLFW_KEY_ENTER,
                scan_code: 0,
                modifiers: 0,
                repeat: false,
            }),
            TextFieldAction::Submitted
        );
    }

    #[test]
    fn text_field_deletes_unicode_scalars_and_enforces_byte_limit() {
        let mut field = TextField::new(rect(), 4).unwrap();
        field.set_value("😀").unwrap();
        assert_eq!(
            field.handle_input(ScreenInputEvent::MouseButtonPressed {
                mouse_x: 20.0,
                mouse_y: 25.0,
                button: 0,
                modifiers: 0,
            }),
            TextFieldAction::Consumed
        );
        assert_eq!(
            field.handle_input(ScreenInputEvent::KeyPressed {
                key_code: GLFW_KEY_BACKSPACE,
                scan_code: 0,
                modifiers: 0,
                repeat: false,
            }),
            TextFieldAction::Changed
        );
        assert_eq!(field.value(), "");
        for unit in [0xd83d, 0xde00] {
            field.handle_input(ScreenInputEvent::CharacterTyped {
                code_unit: unit,
                modifiers: 0,
            });
        }
        assert_eq!(field.value(), "😀");
        assert_eq!(
            field.handle_input(ScreenInputEvent::CharacterTyped {
                code_unit: b'a' as u16,
                modifiers: 0,
            }),
            TextFieldAction::Consumed
        );
        assert_eq!(field.value(), "😀");
        assert!(field.max_bytes <= MAX_TEXT_FIELD_BYTES);
    }

    #[test]
    fn field_focus_and_text_limits_are_bounded() {
        assert_eq!(
            WidgetRect::new(1.0, 2.0, f32::INFINITY, 10.0)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert!(TextField::new(rect(), MAX_TEXT_FIELD_BYTES + 1).is_err());
        let mut field = TextField::new(rect(), 4).unwrap();
        field.set_value("😀").unwrap();
        assert_eq!(
            field.set_value("😀a").unwrap_err().code(),
            ErrorCode::PayloadTooLarge
        );
        assert!(!field.is_focused());
    }
}
