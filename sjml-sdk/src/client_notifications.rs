//! Bounded, client-only vanilla toast notifications.
//!
//! A module must declare and receive `client.ui.notify`. Notifications execute on the physical
//! client main thread; server and static-registration contexts fail closed. Repeated notifications
//! from one module update its currently visible or queued toast instead of growing an unbounded queue.

use crate::{ErrorCode, SjmlError, SjmlResult};

/// Maximum UTF-8 byte length for a toast title.
pub const MAX_NOTIFICATION_TITLE_BYTES: usize = 128;
/// Maximum UTF-8 byte length for a toast body.
pub const MAX_NOTIFICATION_BODY_BYTES: usize = 128;
/// Minimum display duration, in Minecraft ticks (20 ticks per second).
pub const MIN_NOTIFICATION_DURATION_TICKS: u32 = 20;
/// Maximum display duration, in Minecraft ticks.
pub const MAX_NOTIFICATION_DURATION_TICKS: u32 = 600;

/// Visual emphasis for a toast notification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum NotificationKind {
    Info = 0,
    Success = 1,
    Warning = 2,
    Error = 3,
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_client_ui_v1")]
extern "C" {
    #[link_name = "notify"]
    fn imported_notify(
        title_pointer: i32,
        title_length: i32,
        body_pointer: i32,
        body_length: i32,
        kind: i32,
        duration_ticks: i32,
    ) -> i32;
}

/// Displays a vanilla-style toast on the local client.
///
/// `title` and `body` are literal text, not translation keys or formatted components. They must
/// be nonblank, contain no control characters, and fit their UTF-8 bounds. The host displays one
/// toast per runtime module; another call from that module updates its pending toast. Duration is
/// between 20 and 600 ticks (1–30 seconds), measured from the toast's first render. The Minecraft
/// notification-speed preference does not change it. `Info`, `Success`, `Warning`, and `Error`
/// use distinct title colors.
///
/// Requires the declared and administrator-granted `client.ui.notify` capability. This is a
/// presentation API only and cannot affect server-authoritative game state.
pub fn notify(
    title: &str,
    body: &str,
    kind: NotificationKind,
    duration_ticks: u32,
) -> SjmlResult<()> {
    validate_notification(title, body, duration_ticks)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: both borrowed UTF-8 slices stay alive through the synchronous host call, which
        // copies them before returning. Their lengths were checked against the ABI bounds above.
        let status = unsafe {
            imported_notify(
                title.as_ptr() as usize as u32 as i32,
                title.len() as i32,
                body.as_ptr() as usize as u32 as i32,
                body.len() as i32,
                kind as u32 as i32,
                duration_ticks as i32,
            )
        };
        if status == 0 {
            Ok(())
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected the client notification",
            ))
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = kind;
        Err(SjmlError::new(
            ErrorCode::UnknownErrorCode,
            "client notifications are available only to wasm32 guests",
        ))
    }
}

fn validate_notification(title: &str, body: &str, duration_ticks: u32) -> SjmlResult<()> {
    validate_text(title, MAX_NOTIFICATION_TITLE_BYTES, "title")?;
    validate_text(body, MAX_NOTIFICATION_BODY_BYTES, "body")?;
    if !(MIN_NOTIFICATION_DURATION_TICKS..=MAX_NOTIFICATION_DURATION_TICKS)
        .contains(&duration_ticks)
    {
        return Err(invalid(
            "notification duration must be between 20 and 600 ticks",
        ));
    }
    Ok(())
}

fn validate_text(value: &str, max_bytes: usize, label: &str) -> SjmlResult<()> {
    if value.is_empty() || value.trim().is_empty() || value.len() > max_bytes {
        return Err(invalid(format!(
            "notification {label} must be nonblank and at most {max_bytes} UTF-8 bytes"
        )));
    }
    if value.chars().any(char::is_control) {
        return Err(invalid(format!(
            "notification {label} must not contain control characters"
        )));
    }
    Ok(())
}

fn invalid(message: impl Into<String>) -> SjmlError {
    SjmlError::new(ErrorCode::InvalidArgument, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_bounds_and_display_duration() {
        assert!(validate_notification("Title", "Body", 20).is_ok());
        assert!(validate_notification("Title", "Body", 600).is_ok());
        assert_eq!(
            validate_notification("Title", "Body", 19)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            validate_notification("Title", "Body", 601)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            validate_notification("Title", &"x".repeat(MAX_NOTIFICATION_BODY_BYTES + 1), 40)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn rejects_blank_and_control_text_but_accepts_unicode() {
        assert!(validate_notification("Привет", "Мир", 40).is_ok());
        assert_eq!(
            validate_notification("  ", "Body", 40).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            validate_notification("Title", "two\nlines", 40)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn native_call_fails_closed_after_input_validation() {
        let error = notify("Title", "Body", NotificationKind::Info, 40).unwrap_err();
        assert_eq!(error.code(), ErrorCode::UnknownErrorCode);
        assert_eq!(
            notify("", "Body", NotificationKind::Info, 40)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }
}
