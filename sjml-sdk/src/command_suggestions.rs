//! Bounded synchronous completions for manifest-declared SJML commands.

use crate::{Decoder, ErrorCode, SjmlError, SjmlResult, StableDecode, StableEncode};

/// Maximum number of suggestions accepted for one Brigadier request.
pub const MAX_COMMAND_SUGGESTIONS: usize = 64;
/// Maximum UTF-8 byte length of one suggested command argument.
pub const MAX_COMMAND_SUGGESTION_BYTES: usize = 256;
/// Maximum UTF-8 byte length of the unfinished argument supplied by Brigadier.
pub const MAX_COMMAND_SUGGESTION_PARTIAL_BYTES: usize = 65_536;
/// Maximum UTF-8 byte length of partial text from a physical-client command.
pub const MAX_CLIENT_COMMAND_SUGGESTION_PARTIAL_BYTES: usize = 256;
/// Manifest and local-grant capability required for physical-client suggestions.
pub const CLIENT_COMMAND_SUGGEST_CAPABILITY: &str = "client.command.suggest";

/// One already parsed command argument provided to a dynamic suggestion callback.
#[derive(Clone, Debug, Eq, PartialEq, StableDecode, StableEncode)]
#[sjml(version = 1)]
pub struct CommandSuggestionArgument {
    /// Argument name from the module manifest.
    pub name: String,
    /// Manifest type: `word`, `string`, `greedy_string`, `integer`, or `boolean`.
    pub kind: String,
    /// Canonical textual representation of the parsed value.
    pub value: String,
}

/// Context passed to a synchronous manifest-declared command suggestion callback.
#[derive(Clone, Debug, Eq, PartialEq, StableDecode, StableEncode)]
#[sjml(version = 1)]
pub struct CommandSuggestionRequest {
    /// Command callback ID, such as `players.teleport`.
    pub command_name: String,
    /// UUID of the invoking player, or `None` when suggestions came from the console.
    pub source_player_id: Option<String>,
    /// Zero-based index of the argument being suggested.
    pub argument_index: u32,
    /// Argument name from the module manifest.
    pub argument_name: String,
    /// Unfinished text currently being typed for this argument.
    pub partial: String,
    /// Arguments parsed before the argument being suggested.
    pub previous_arguments: Vec<CommandSuggestionArgument>,
}

/// Context passed to a synchronous physical-client command-suggestion callback.
///
/// This request intentionally contains no server command source, player identity,
/// world, or live Minecraft object.
#[derive(Clone, Debug, Eq, PartialEq, StableDecode, StableEncode)]
#[sjml(version = 1)]
pub struct ClientCommandSuggestionRequest {
    /// Command callback ID declared in `[[modules.client_commands]]`.
    pub command_name: String,
    /// Zero-based index of the argument being suggested.
    pub argument_index: u32,
    /// Argument name from the module manifest.
    pub argument_name: String,
    /// Unfinished local text, limited by the host to 256 UTF-8 bytes.
    pub partial: String,
    /// Arguments parsed before the argument being suggested.
    pub previous_arguments: Vec<CommandSuggestionArgument>,
}

/// Bounded candidate values returned to Brigadier for one argument.
#[derive(Clone, Debug, Eq, PartialEq, StableEncode)]
#[sjml(version = 1)]
pub struct CommandSuggestions {
    values: Vec<String>,
}

impl CommandSuggestions {
    /// Creates validated unique suggestions, preserving their original order.
    pub fn new(values: impl IntoIterator<Item = impl Into<String>>) -> SjmlResult<Self> {
        let mut result = Vec::new();
        for value in values {
            if result.len() == MAX_COMMAND_SUGGESTIONS {
                return Err(SjmlError::new(
                    ErrorCode::PayloadTooLarge,
                    format!("a command callback may return at most {MAX_COMMAND_SUGGESTIONS} suggestions"),
                ));
            }
            let value = value.into();
            if value.is_empty()
                || value.len() > MAX_COMMAND_SUGGESTION_BYTES
                || value.chars().any(char::is_control)
            {
                return Err(SjmlError::new(
                    ErrorCode::InvalidArgument,
                    format!(
                        "a command suggestion must contain 1–{MAX_COMMAND_SUGGESTION_BYTES} UTF-8 bytes without control characters"
                    ),
                ));
            }
            if !result.contains(&value) {
                result.push(value);
            }
        }
        Ok(Self { values: result })
    }

    /// Returns an empty suggestion list.
    pub const fn empty() -> Self {
        Self { values: Vec::new() }
    }

    /// Returns the validated suggestion values.
    pub fn values(&self) -> &[String] {
        &self.values
    }
}

impl StableDecode for CommandSuggestions {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        let version = decoder.read_u32()?;
        if version != 1 {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                format!("unsupported command-suggestions schema version {version}"),
            ));
        }
        let count = decoder.read_length(MAX_COMMAND_SUGGESTIONS)?;
        let mut values = Vec::with_capacity(count);
        for _ in 0..count {
            values.push(String::decode(decoder)?);
        }
        Self::new(values)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ClientCommandSuggestionRequest, CommandSuggestionArgument, CommandSuggestionRequest,
        CommandSuggestions, MAX_CLIENT_COMMAND_SUGGESTION_PARTIAL_BYTES, MAX_COMMAND_SUGGESTIONS,
        MAX_COMMAND_SUGGESTION_BYTES,
    };
    use crate::{from_bytes, to_bytes, ErrorCode};

    #[test]
    fn request_and_response_round_trip_the_versioned_stable_layout() {
        let request = CommandSuggestionRequest {
            command_name: "players.teleport".into(),
            source_player_id: Some("00000000-0000-0000-0000-000000000001".into()),
            argument_index: 1,
            argument_name: "player".into(),
            partial: "al".into(),
            previous_arguments: vec![CommandSuggestionArgument {
                name: "dimension".into(),
                kind: "word".into(),
                value: "minecraft:overworld".into(),
            }],
        };
        let request_bytes = to_bytes(&request).unwrap();
        assert_eq!(request, from_bytes(&request_bytes).unwrap());
        assert_eq!(
            u32::from_le_bytes(request_bytes[..4].try_into().unwrap()),
            1
        );

        let response = CommandSuggestions::new(["alex", "alex", "alexa"]).unwrap();
        assert_eq!(response.values(), &["alex", "alexa"]);
        assert_eq!(response, from_bytes(&to_bytes(&response).unwrap()).unwrap());
    }

    #[test]
    fn suggestions_enforce_count_size_and_text_bounds() {
        let too_many = (0..=MAX_COMMAND_SUGGESTIONS).map(|index| format!("player{index}"));
        assert_eq!(
            CommandSuggestions::new(too_many).unwrap_err().code(),
            ErrorCode::PayloadTooLarge
        );
        assert_eq!(
            CommandSuggestions::new(["x".repeat(MAX_COMMAND_SUGGESTION_BYTES + 1)])
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert!(CommandSuggestions::new(["line\nbreak"]).is_err());
        assert!(CommandSuggestions::new([""]).is_err());
    }

    #[test]
    fn client_request_uses_local_detached_arguments_and_bounded_partial_input() {
        let request = ClientCommandSuggestionRequest {
            command_name: "tools.lookup".into(),
            argument_index: 1,
            argument_name: "target".into(),
            partial: "Al".into(),
            previous_arguments: vec![CommandSuggestionArgument {
                name: "dimension".into(),
                kind: "word".into(),
                value: "overworld".into(),
            }],
        };
        let encoded = to_bytes(&request).unwrap();
        assert_eq!(request, from_bytes(&encoded).unwrap());
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_u32.to_le_bytes());
        append_string(&mut expected, "tools.lookup");
        expected.extend_from_slice(&1_u32.to_le_bytes());
        append_string(&mut expected, "target");
        append_string(&mut expected, "Al");
        expected.extend_from_slice(&1_u32.to_le_bytes());
        expected.extend_from_slice(&1_u32.to_le_bytes());
        append_string(&mut expected, "dimension");
        append_string(&mut expected, "word");
        append_string(&mut expected, "overworld");
        assert_eq!(encoded, expected);
        assert_eq!(MAX_CLIENT_COMMAND_SUGGESTION_PARTIAL_BYTES, 256);
    }

    fn append_string(output: &mut Vec<u8>, value: &str) {
        output.extend_from_slice(&(value.len() as u32).to_le_bytes());
        output.extend_from_slice(value.as_bytes());
    }
}
