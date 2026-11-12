//! Bounded access to entries in vanilla scoreboard objectives that already exist on the logical server.
//! Reading requires `scoreboard.read`; changing a value requires `scoreboard.write`. The guest cannot create
//! objectives, teams, display slots, or Minecraft score-holder objects.

use crate::{ErrorCode, SjmlError, SjmlResult};

/// Maximum UTF-8 bytes accepted for an existing vanilla objective name.
pub const MAX_SCOREBOARD_OBJECTIVE_BYTES: usize = 16;
/// Maximum UTF-8 bytes accepted for a scoreboard entry name.
pub const MAX_SCOREBOARD_ENTRY_BYTES: usize = 40;
/// Maximum score magnitude accepted by the host and SDK.
pub const MAX_SCORE_VALUE: i32 = 1_000_000_000;
/// Fixed ABI-v1 output size for [`score`].
pub const SCORE_RESPONSE_BYTES: usize = 12;
#[cfg(any(target_arch = "wasm32", test))]
const SCORE_RESPONSE_SCHEMA: u32 = 1;

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_scoreboard_v1")]
extern "C" {
    #[link_name = "score"]
    fn imported_score(
        objective_pointer: i32,
        objective_length: i32,
        entry_pointer: i32,
        entry_length: i32,
        output_pointer: i32,
        output_capacity: i32,
    ) -> i32;
    #[link_name = "set_score"]
    fn imported_set_score(
        objective_pointer: i32,
        objective_length: i32,
        entry_pointer: i32,
        entry_length: i32,
        value: i32,
    ) -> i32;
}

/// Reads one named entry from an existing objective. Returns `None` until that entry has a score.
pub fn score(objective: &str, entry: &str) -> SjmlResult<Option<i32>> {
    validate_names(objective, entry)?;
    #[cfg(target_arch = "wasm32")]
    {
        let mut output = [0_u8; SCORE_RESPONSE_BYTES];
        // SAFETY: both UTF-8 slices and the fixed output array remain live for this synchronous host call.
        let status = unsafe {
            imported_score(
                objective.as_ptr() as usize as u32 as i32,
                objective.len() as u32 as i32,
                entry.as_ptr() as usize as u32 as i32,
                entry.len() as u32 as i32,
                output.as_mut_ptr() as usize as u32 as i32,
                output.len() as u32 as i32,
            )
        };
        status_result(status, "host rejected the scoreboard score read")?;
        decode_score_response(&output)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported_target())
    }
}

/// Sets an entry in an existing writable objective. It does not create the objective or other scoreboard data.
pub fn set_score(objective: &str, entry: &str, value: i32) -> SjmlResult<()> {
    validate_names(objective, entry)?;
    if !(-MAX_SCORE_VALUE..=MAX_SCORE_VALUE).contains(&value) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            format!("score value must be within ±{MAX_SCORE_VALUE}"),
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: both UTF-8 slices remain live for this synchronous host call.
        let status = unsafe {
            imported_set_score(
                objective.as_ptr() as usize as u32 as i32,
                objective.len() as u32 as i32,
                entry.as_ptr() as usize as u32 as i32,
                entry.len() as u32 as i32,
                value,
            )
        };
        status_result(status, "host rejected the scoreboard score update")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported_target())
    }
}

fn validate_names(objective: &str, entry: &str) -> SjmlResult<()> {
    if objective.is_empty()
        || objective.len() > MAX_SCOREBOARD_OBJECTIVE_BYTES
        || !objective
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_.+-".contains(&byte))
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "objective names must be 1–16 ASCII letters, digits, '.', '_', '+', or '-'",
        ));
    }
    if entry.is_empty()
        || entry.chars().all(char::is_whitespace)
        || entry.starts_with('#')
        || entry.len() > MAX_SCOREBOARD_ENTRY_BYTES
        || entry.chars().count() > MAX_SCOREBOARD_ENTRY_BYTES
        || entry.chars().any(char::is_control)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "score entries must be 1–40 UTF-8 bytes/characters without controls or a leading '#'",
        ));
    }
    Ok(())
}

#[cfg(any(target_arch = "wasm32", test))]
fn decode_score_response(bytes: &[u8]) -> SjmlResult<Option<i32>> {
    if bytes.len() != SCORE_RESPONSE_BYTES {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "scoreboard score response must contain exactly twelve bytes",
        ));
    }
    let schema = u32::from_le_bytes(bytes[0..4].try_into().expect("fixed-width score schema"));
    if schema != SCORE_RESPONSE_SCHEMA {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "host returned an unsupported scoreboard score schema",
        ));
    }
    let present = u32::from_le_bytes(bytes[4..8].try_into().expect("fixed-width score presence"));
    let value = i32::from_le_bytes(bytes[8..12].try_into().expect("fixed-width score value"));
    match present {
        0 if value == 0 => Ok(None),
        1 if (-MAX_SCORE_VALUE..=MAX_SCORE_VALUE).contains(&value) => Ok(Some(value)),
        _ => Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "host returned an invalid scoreboard score response",
        )),
    }
}

#[cfg(target_arch = "wasm32")]
fn status_result(status: i32, message: &str) -> SjmlResult<()> {
    if status == 0 {
        Ok(())
    } else {
        Err(SjmlError::from_abi_code(status as u32, message))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn unsupported_target() -> SjmlError {
    SjmlError::new(
        ErrorCode::UnknownErrorCode,
        "scoreboard imports require the wasm32-unknown-unknown target",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scores_decode_the_fixed_little_endian_optional_contract() {
        assert_eq!(
            decode_score_response(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]).unwrap(),
            None
        );
        let mut bytes = Vec::from([1_u8, 0, 0, 0, 1, 0, 0, 0]);
        bytes.extend_from_slice(&(-17_i32).to_le_bytes());
        assert_eq!(decode_score_response(&bytes).unwrap(), Some(-17));
    }

    #[test]
    fn names_and_score_values_are_checked_before_host_access() {
        for objective in ["", "name with space", "objective_name_that_is_long"] {
            assert_eq!(
                score(objective, "Alice").unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
        }
        let long_entry = "x".repeat(MAX_SCOREBOARD_ENTRY_BYTES + 1);
        for entry in ["", "   ", "#hidden", "has\ncontrol", long_entry.as_str()] {
            assert_eq!(
                score("kills", entry).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
        }
        assert_eq!(
            set_score("kills", "Alice", MAX_SCORE_VALUE + 1)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            set_score("kills", "Alice", 1).unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn malformed_schema_presence_and_trailing_bytes_are_rejected() {
        assert_eq!(
            decode_score_response(&[2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            decode_score_response(&[1, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0])
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            decode_score_response(&[1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0])
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            decode_score_response(&[0; 13]).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }
}
