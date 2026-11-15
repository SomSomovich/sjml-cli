//! Detached observation of a logical-server difficulty change.
//!
//! Subscribe with `events::subscribe("world.difficulty_change", EventPriority::Normal)` and a
//! typed `#[event(name = "world.difficulty_change")]` callback returning `SjmlResult<()>`. The
//! notification is observational: it cannot change or cancel the new difficulty. The host fires it
//! on the owning server thread when the current server's difficulty changes, and invokes at most
//! [`MAX_CALLBACKS_PER_EVENT`] callbacks per change.
//!
//! The host only dispatches this event for logical-server contexts; the underlying NeoForge hook
//! carries no level, and the notification contains only the old and new difficulty names.

use sjml::{StableDecode, StableEncode};

/// Event name used with [`crate::events::subscribe`].
pub const EVENT_NAME: &str = "world.difficulty_change";
/// Generated ABI export name for the typed difficulty-change callback.
pub const EXPORT_NAME: &str = "sjml_v1_event_world.difficulty_change";
/// Maximum UTF-8 byte length of a difficulty field.
pub const MAX_STRING_BYTES: usize = 64;
/// Maximum encoded bytes for one detached difficulty-change event.
pub const MAX_PAYLOAD_BYTES: usize = 256;
/// Maximum guest callbacks invoked for one difficulty-change event.
pub const MAX_CALLBACKS_PER_EVENT: usize = 64;

/// Detached difficulty-change notification. It contains no level, server, or host handle.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct WorldDifficultyChange {
    /// Difficulty after the change: `peaceful`, `easy`, `normal`, or `hard`.
    pub new_difficulty: String,
    /// Difficulty before the change: `peaceful`, `easy`, `normal`, or `hard`.
    pub old_difficulty: String,
}

#[cfg(test)]
mod tests {
    use super::{
        WorldDifficultyChange, EVENT_NAME, EXPORT_NAME, MAX_CALLBACKS_PER_EVENT, MAX_PAYLOAD_BYTES,
        MAX_STRING_BYTES,
    };
    use sjml::{from_bytes, to_bytes, ErrorCode};

    fn event() -> WorldDifficultyChange {
        WorldDifficultyChange {
            new_difficulty: "hard".into(),
            old_difficulty: "normal".into(),
        }
    }

    #[test]
    fn wire_bytes_match_schema_v1_field_order() {
        let original = event();
        let bytes = to_bytes(&original).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_u32.to_le_bytes());
        expected.extend_from_slice(&(original.new_difficulty.len() as u32).to_le_bytes());
        expected.extend_from_slice(original.new_difficulty.as_bytes());
        expected.extend_from_slice(&(original.old_difficulty.len() as u32).to_le_bytes());
        expected.extend_from_slice(original.old_difficulty.as_bytes());
        assert_eq!(bytes, expected);
        assert!(bytes.len() <= MAX_PAYLOAD_BYTES);
    }

    #[test]
    fn stable_round_trip_preserves_both_difficulties() {
        for (new_difficulty, old_difficulty) in [
            ("peaceful", "hard"),
            ("easy", "peaceful"),
            ("normal", "easy"),
            ("hard", "normal"),
        ] {
            let original = WorldDifficultyChange {
                new_difficulty: new_difficulty.into(),
                old_difficulty: old_difficulty.into(),
            };
            let bytes = to_bytes(&original).unwrap();
            assert_eq!(
                u32::from_le_bytes(bytes[0..4].try_into().unwrap()),
                1,
                "the schema version must be the first little-endian word"
            );
            assert_eq!(
                from_bytes::<WorldDifficultyChange>(&bytes).unwrap(),
                original
            );
        }
    }

    #[test]
    fn codec_rejects_unsupported_schema_and_trailing_bytes() {
        let bytes = to_bytes(&event()).unwrap();

        let mut bad_schema = bytes.clone();
        bad_schema[0..4].copy_from_slice(&2_u32.to_le_bytes());
        assert_eq!(
            from_bytes::<WorldDifficultyChange>(&bad_schema)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );

        let mut trailing = bytes;
        trailing.push(0);
        assert!(from_bytes::<WorldDifficultyChange>(&trailing).is_err());
    }

    #[test]
    fn published_limits_match_the_host_contract() {
        assert_eq!("world.difficulty_change", EVENT_NAME);
        assert_eq!("sjml_v1_event_world.difficulty_change", EXPORT_NAME);
        assert_eq!(64, MAX_STRING_BYTES);
        assert_eq!(256, MAX_PAYLOAD_BYTES);
        assert_eq!(64, MAX_CALLBACKS_PER_EVENT);
    }
}
