//! Bounded access to the active logical-server scoreboard. Objective score access is limited to existing
//! objectives. Module-owned teams use local IDs and cannot access vanilla or another module's teams.

use crate::{ErrorCode, SjmlError, SjmlResult};

/// Maximum UTF-8 bytes accepted for an existing vanilla objective name.
pub const MAX_SCOREBOARD_OBJECTIVE_BYTES: usize = 16;
/// Maximum UTF-8 bytes accepted for a scoreboard entry name.
pub const MAX_SCOREBOARD_ENTRY_BYTES: usize = 40;
/// Maximum score magnitude accepted by the host and SDK.
pub const MAX_SCORE_VALUE: i32 = 1_000_000_000;
/// Fixed ABI-v1 output size for [`score`].
pub const SCORE_RESPONSE_BYTES: usize = 12;
/// Maximum UTF-8 bytes accepted for module-owned team display text.
pub const MAX_TEAM_TEXT_BYTES: usize = 128;
/// Maximum Unicode scalar values accepted for module-owned team display text.
pub const MAX_TEAM_TEXT_CODE_POINTS: usize = 64;
/// Maximum number of bytes returned by the bounded team snapshot ABI.
pub const TEAM_RESPONSE_MAX_BYTES: usize = 1_024;
#[cfg(any(target_arch = "wasm32", test))]
const SCORE_RESPONSE_SCHEMA: u32 = 1;
#[cfg(any(target_arch = "wasm32", test))]
const TEAM_RESPONSE_SCHEMA: u32 = 1;

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
    #[link_name = "team"]
    fn imported_team(
        id_pointer: i32,
        id_length: i32,
        output_pointer: i32,
        output_capacity: i32,
    ) -> i32;
    #[link_name = "create_team"]
    fn imported_create_team(id_pointer: i32, id_length: i32) -> i32;
    #[link_name = "remove_team"]
    fn imported_remove_team(id_pointer: i32, id_length: i32) -> i32;
    #[link_name = "set_team_options"]
    fn imported_set_team_options(
        id_pointer: i32,
        id_length: i32,
        display_pointer: i32,
        display_length: i32,
        prefix_pointer: i32,
        prefix_length: i32,
        suffix_pointer: i32,
        suffix_length: i32,
        color: i32,
        name_tag_visibility: i32,
        death_message_visibility: i32,
        collision_rule: i32,
        friendly_fire: i32,
        see_friendly_invisibles: i32,
    ) -> i32;
    #[link_name = "add_player_to_team"]
    fn imported_add_player_to_team(
        id_pointer: i32,
        id_length: i32,
        uuid_pointer: i32,
        uuid_length: i32,
    ) -> i32;
    #[link_name = "remove_player_from_team"]
    fn imported_remove_player_from_team(
        id_pointer: i32,
        id_length: i32,
        uuid_pointer: i32,
        uuid_length: i32,
    ) -> i32;
}

/// Vanilla formatting color code used by module-owned scoreboard teams.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum TeamColor {
    Reset = 0,
    Black = 1,
    DarkBlue = 2,
    DarkGreen = 3,
    DarkAqua = 4,
    DarkRed = 5,
    DarkPurple = 6,
    Gold = 7,
    Gray = 8,
    DarkGray = 9,
    Blue = 10,
    Green = 11,
    Aqua = 12,
    Red = 13,
    LightPurple = 14,
    Yellow = 15,
    White = 16,
}

/// Name-tag or death-message visibility for a team.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum TeamVisibility {
    Always = 0,
    Never = 1,
    HideForOtherTeams = 2,
    HideForOwnTeam = 3,
}

/// Vanilla entity collision behavior for a team.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum TeamCollisionRule {
    Always = 0,
    Never = 1,
    PushOtherTeams = 2,
    PushOwnTeam = 3,
}

/// Literal display and gameplay options for an owned team.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeamOptions {
    pub display_name: String,
    pub prefix: String,
    pub suffix: String,
    pub color: TeamColor,
    pub name_tag_visibility: TeamVisibility,
    pub death_message_visibility: TeamVisibility,
    pub collision_rule: TeamCollisionRule,
    pub friendly_fire: bool,
    pub see_friendly_invisibles: bool,
}

/// Detached values read from an owned vanilla scoreboard team.
pub type TeamSnapshot = TeamOptions;

/// Creates an empty module-owned team, or succeeds if this module already owns it.
pub fn create_team(id: &str) -> SjmlResult<()> {
    validate_team_id(id)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the UTF-8 ID slice remains valid for the synchronous host call.
        let status = unsafe {
            imported_create_team(id.as_ptr() as usize as u32 as i32, id.len() as u32 as i32)
        };
        status_result(status, "host rejected the scoreboard team creation")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported_target())
    }
}

/// Removes this module's owned team and its memberships. Foreign teams are never changed.
pub fn remove_team(id: &str) -> SjmlResult<()> {
    validate_team_id(id)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the UTF-8 ID slice remains valid for the synchronous host call.
        let status = unsafe {
            imported_remove_team(id.as_ptr() as usize as u32 as i32, id.len() as u32 as i32)
        };
        status_result(status, "host rejected the scoreboard team removal")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported_target())
    }
}

/// Sets literal text and validated vanilla behavior on an existing owned team.
pub fn set_team_options(id: &str, options: &TeamOptions) -> SjmlResult<()> {
    validate_team_id(id)?;
    validate_team_text(&options.display_name, false, "team display name")?;
    validate_team_text(&options.prefix, true, "team prefix")?;
    validate_team_text(&options.suffix, true, "team suffix")?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: all input slices remain valid for this synchronous host call.
        let status = unsafe {
            imported_set_team_options(
                id.as_ptr() as usize as u32 as i32,
                id.len() as u32 as i32,
                options.display_name.as_ptr() as usize as u32 as i32,
                options.display_name.len() as u32 as i32,
                options.prefix.as_ptr() as usize as u32 as i32,
                options.prefix.len() as u32 as i32,
                options.suffix.as_ptr() as usize as u32 as i32,
                options.suffix.len() as u32 as i32,
                options.color as u32 as i32,
                options.name_tag_visibility as u32 as i32,
                options.death_message_visibility as u32 as i32,
                options.collision_rule as u32 as i32,
                i32::from(options.friendly_fire),
                i32::from(options.see_friendly_invisibles),
            )
        };
        status_result(status, "host rejected the scoreboard team options")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported_target())
    }
}

/// Reads a bounded snapshot of an existing team owned by this module.
pub fn team(id: &str) -> SjmlResult<TeamSnapshot> {
    validate_team_id(id)?;
    #[cfg(target_arch = "wasm32")]
    {
        let mut output = [0_u8; TEAM_RESPONSE_MAX_BYTES];
        // SAFETY: input and output slices remain valid for the synchronous host call.
        let status = unsafe {
            imported_team(
                id.as_ptr() as usize as u32 as i32,
                id.len() as u32 as i32,
                output.as_mut_ptr() as usize as u32 as i32,
                output.len() as u32 as i32,
            )
        };
        status_result(status, "host rejected the scoreboard team read")?;
        decode_team_response(&output)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported_target())
    }
}

/// Adds an online player to an owned team if the player has no foreign team assignment.
pub fn add_player_to_team(id: &str, player_uuid: &str) -> SjmlResult<()> {
    validate_team_id(id)?;
    validate_player_uuid(player_uuid)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: both input slices remain valid for the synchronous host call.
        let status = unsafe {
            imported_add_player_to_team(
                id.as_ptr() as usize as u32 as i32,
                id.len() as u32 as i32,
                player_uuid.as_ptr() as usize as u32 as i32,
                player_uuid.len() as u32 as i32,
            )
        };
        status_result(
            status,
            "host rejected the scoreboard team membership update",
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported_target())
    }
}

/// Removes an online player only when currently assigned to this owned team.
pub fn remove_player_from_team(id: &str, player_uuid: &str) -> SjmlResult<()> {
    validate_team_id(id)?;
    validate_player_uuid(player_uuid)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: both input slices remain valid for the synchronous host call.
        let status = unsafe {
            imported_remove_player_from_team(
                id.as_ptr() as usize as u32 as i32,
                id.len() as u32 as i32,
                player_uuid.as_ptr() as usize as u32 as i32,
                player_uuid.len() as u32 as i32,
            )
        };
        status_result(
            status,
            "host rejected the scoreboard team membership update",
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported_target())
    }
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

fn validate_team_id(id: &str) -> SjmlResult<()> {
    if id.is_empty()
        || id.len() > 64
        || !id.as_bytes()[0].is_ascii_lowercase()
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_-".contains(&byte))
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "module team IDs must be 1–64 lowercase ASCII letters, digits, '_' or '-', starting with a letter",
        ));
    }
    Ok(())
}

fn validate_team_text(value: &str, allow_empty: bool, label: &str) -> SjmlResult<()> {
    if (!allow_empty && value.is_empty())
        || value.len() > MAX_TEAM_TEXT_BYTES
        || value.chars().count() > MAX_TEAM_TEXT_CODE_POINTS
        || value.chars().any(char::is_control)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            format!("{label} must be at most 128 UTF-8 bytes/64 characters without controls"),
        ));
    }
    Ok(())
}

fn validate_player_uuid(uuid: &str) -> SjmlResult<()> {
    let bytes = uuid.as_bytes();
    let valid = bytes.len() == 36
        && [8, 13, 18, 23].iter().all(|index| bytes[*index] == b'-')
        && bytes.iter().enumerate().all(|(index, byte)| {
            [8, 13, 18, 23].contains(&index)
                || byte.is_ascii_digit()
                || (b'a'..=b'f').contains(byte)
        });
    if !valid {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player UUID must use canonical lowercase 8-4-4-4-12 hexadecimal form",
        ));
    }
    Ok(())
}

#[cfg(any(target_arch = "wasm32", test))]
fn decode_team_response(bytes: &[u8]) -> SjmlResult<TeamSnapshot> {
    if bytes.len() < 4 || bytes.len() > TEAM_RESPONSE_MAX_BYTES {
        return Err(invalid_team_response("team response has an invalid size"));
    }
    let mut decoder = TeamResponseDecoder { bytes, offset: 0 };
    if decoder.u32()? != TEAM_RESPONSE_SCHEMA {
        return Err(invalid_team_response(
            "host returned an unsupported team response schema",
        ));
    }
    let display_name = decoder.string()?;
    let prefix = decoder.string()?;
    let suffix = decoder.string()?;
    validate_team_text(&display_name, false, "team display name")?;
    validate_team_text(&prefix, true, "team prefix")?;
    validate_team_text(&suffix, true, "team suffix")?;
    let color = match decoder.u32()? {
        0 => TeamColor::Reset,
        1 => TeamColor::Black,
        2 => TeamColor::DarkBlue,
        3 => TeamColor::DarkGreen,
        4 => TeamColor::DarkAqua,
        5 => TeamColor::DarkRed,
        6 => TeamColor::DarkPurple,
        7 => TeamColor::Gold,
        8 => TeamColor::Gray,
        9 => TeamColor::DarkGray,
        10 => TeamColor::Blue,
        11 => TeamColor::Green,
        12 => TeamColor::Aqua,
        13 => TeamColor::Red,
        14 => TeamColor::LightPurple,
        15 => TeamColor::Yellow,
        16 => TeamColor::White,
        _ => return Err(invalid_team_response("host returned an unknown team color")),
    };
    let name_tag_visibility = decode_visibility(decoder.u32()?)?;
    let death_message_visibility = decode_visibility(decoder.u32()?)?;
    let collision_rule = match decoder.u32()? {
        0 => TeamCollisionRule::Always,
        1 => TeamCollisionRule::Never,
        2 => TeamCollisionRule::PushOtherTeams,
        3 => TeamCollisionRule::PushOwnTeam,
        _ => {
            return Err(invalid_team_response(
                "host returned an unknown collision rule",
            ))
        }
    };
    let friendly_fire = decoder.boolean()?;
    let see_friendly_invisibles = decoder.boolean()?;
    if bytes[decoder.offset..].iter().any(|byte| *byte != 0) {
        return Err(invalid_team_response(
            "team response contains trailing data",
        ));
    }
    Ok(TeamSnapshot {
        display_name,
        prefix,
        suffix,
        color,
        name_tag_visibility,
        death_message_visibility,
        collision_rule,
        friendly_fire,
        see_friendly_invisibles,
    })
}

#[cfg(any(target_arch = "wasm32", test))]
fn decode_visibility(value: u32) -> SjmlResult<TeamVisibility> {
    match value {
        0 => Ok(TeamVisibility::Always),
        1 => Ok(TeamVisibility::Never),
        2 => Ok(TeamVisibility::HideForOtherTeams),
        3 => Ok(TeamVisibility::HideForOwnTeam),
        _ => Err(invalid_team_response(
            "host returned an unknown team visibility",
        )),
    }
}

#[cfg(any(target_arch = "wasm32", test))]
struct TeamResponseDecoder<'a> {
    bytes: &'a [u8],
    offset: usize,
}

#[cfg(any(target_arch = "wasm32", test))]
impl TeamResponseDecoder<'_> {
    fn u32(&mut self) -> SjmlResult<u32> {
        let end = self
            .offset
            .checked_add(4)
            .ok_or_else(|| invalid_team_response("team response length overflow"))?;
        let raw = self
            .bytes
            .get(self.offset..end)
            .ok_or_else(|| invalid_team_response("team response is truncated"))?;
        self.offset = end;
        Ok(u32::from_le_bytes(
            raw.try_into().expect("four-byte team field"),
        ))
    }

    fn string(&mut self) -> SjmlResult<String> {
        let length = self.u32()? as usize;
        if length > MAX_TEAM_TEXT_BYTES {
            return Err(invalid_team_response(
                "host returned an oversized team text value",
            ));
        }
        let end = self
            .offset
            .checked_add(length)
            .ok_or_else(|| invalid_team_response("team response length overflow"))?;
        let raw = self
            .bytes
            .get(self.offset..end)
            .ok_or_else(|| invalid_team_response("team response text is truncated"))?;
        self.offset = end;
        String::from_utf8(raw.to_vec())
            .map_err(|_| invalid_team_response("host returned invalid UTF-8 team text"))
    }

    fn boolean(&mut self) -> SjmlResult<bool> {
        let value = *self
            .bytes
            .get(self.offset)
            .ok_or_else(|| invalid_team_response("team response is truncated"))?;
        self.offset += 1;
        match value {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(invalid_team_response(
                "host returned an invalid team boolean",
            )),
        }
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn invalid_team_response(message: &str) -> SjmlError {
    SjmlError::new(ErrorCode::InvalidArgument, message)
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

    #[test]
    fn module_team_ids_text_and_player_uuids_are_bounded_before_imports() {
        assert_eq!(validate_team_id("main-team").unwrap(), ());
        for invalid in ["", "9team", "Main", "bad.id", "bad team", "équipe"] {
            assert_eq!(
                validate_team_id(invalid).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
        }
        let long_id = format!("a{}", "x".repeat(64));
        assert_eq!(
            validate_team_id(&long_id).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );

        assert!(validate_team_text("Team 🔥", false, "display").is_ok());
        assert!(validate_team_text("", true, "prefix").is_ok());
        assert_eq!(
            validate_team_text("", false, "display").unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            validate_team_text("line\nbreak", true, "prefix")
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        let oversized = "x".repeat(MAX_TEAM_TEXT_BYTES + 1);
        assert_eq!(
            validate_team_text(&oversized, true, "prefix")
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );

        assert!(validate_player_uuid("123e4567-e89b-12d3-a456-426614174000").is_ok());
        assert_eq!(
            validate_player_uuid("123E4567-E89B-12D3-A456-426614174000")
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn team_snapshot_decodes_v1_little_endian_fields_and_rejects_malformed_data() {
        let mut encoded = Vec::new();
        encoded.extend_from_slice(&1_u32.to_le_bytes());
        for value in ["Admins", "[A] ", " ★"] {
            encoded.extend_from_slice(&(value.len() as u32).to_le_bytes());
            encoded.extend_from_slice(value.as_bytes());
        }
        for value in [13_u32, 2, 3, 2] {
            encoded.extend_from_slice(&value.to_le_bytes());
        }
        encoded.extend_from_slice(&[1, 0]);
        encoded.resize(TEAM_RESPONSE_MAX_BYTES, 0);
        assert_eq!(
            decode_team_response(&encoded).unwrap(),
            TeamSnapshot {
                display_name: "Admins".into(),
                prefix: "[A] ".into(),
                suffix: " ★".into(),
                color: TeamColor::Red,
                name_tag_visibility: TeamVisibility::HideForOtherTeams,
                death_message_visibility: TeamVisibility::HideForOwnTeam,
                collision_rule: TeamCollisionRule::PushOtherTeams,
                friendly_fire: true,
                see_friendly_invisibles: false,
            }
        );

        let mut unsupported = encoded.clone();
        unsupported[0] = 2;
        assert_eq!(
            decode_team_response(&unsupported).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        let mut bad_boolean = encoded.clone();
        bad_boolean[46] = 2;
        assert_eq!(
            decode_team_response(&bad_boolean).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        let mut nonzero_trailing = encoded;
        *nonzero_trailing.last_mut().unwrap() = 1;
        assert_eq!(
            decode_team_response(&nonzero_trailing).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn native_team_calls_still_validate_then_report_missing_wasm_host() {
        let options = TeamOptions {
            display_name: "Players".into(),
            prefix: "".into(),
            suffix: "".into(),
            color: TeamColor::Reset,
            name_tag_visibility: TeamVisibility::Always,
            death_message_visibility: TeamVisibility::Always,
            collision_rule: TeamCollisionRule::Always,
            friendly_fire: false,
            see_friendly_invisibles: false,
        };
        assert_eq!(
            create_team("invalid.id").unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            set_team_options("players", &options).unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            add_player_to_team("players", "not-a-uuid")
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }
}
