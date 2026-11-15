//! Detached observation of a logical-server player waking up.
//!
//! Subscribe with `events::subscribe("player.wake_up", EventPriority::Normal)` and a typed
//! `#[event(name = "player.wake_up")]` callback returning `SjmlResult<()>`. The notification is
//! observational: it cannot cancel or change the wake-up. Callbacks run on the owning server
//! thread, and the host invokes at most [`MAX_CALLBACKS_PER_EVENT`] callbacks per wake-up.

use sjml::{StableDecode, StableEncode};

/// Event name used with [`crate::events::subscribe`].
pub const EVENT_NAME: &str = "player.wake_up";
/// Generated ABI export name for the typed wake-up callback.
pub const EXPORT_NAME: &str = "sjml_v1_event_player.wake_up";
/// Maximum UTF-8 byte length of the player UUID and dimension ID.
pub const MAX_STRING_BYTES: usize = 256;
/// Maximum encoded bytes for one detached wake-up event.
pub const MAX_PAYLOAD_BYTES: usize = 1_024;
/// Maximum guest callbacks invoked for one wake-up event.
pub const MAX_CALLBACKS_PER_EVENT: usize = 64;

/// Detached wake-up notification. It contains no `Player`, level, or host handle.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerWakeUp {
    /// Canonical lowercase UUID of the player that woke up.
    pub player_id: String,
    /// Resource location of the dimension containing the player.
    pub dimension_id: String,
    /// Whether vanilla should skip the gradual wake-up animation.
    pub wake_immediately: bool,
    /// Whether the server should be notified of the sleeping-state change.
    pub update_level: bool,
}

#[cfg(test)]
mod tests {
    use super::{
        PlayerWakeUp, EVENT_NAME, EXPORT_NAME, MAX_CALLBACKS_PER_EVENT, MAX_PAYLOAD_BYTES,
        MAX_STRING_BYTES,
    };
    use sjml::{from_bytes, to_bytes, ErrorCode};

    fn event() -> PlayerWakeUp {
        PlayerWakeUp {
            player_id: "123e4567-e89b-12d3-a456-426614174000".into(),
            dimension_id: "minecraft:overworld".into(),
            wake_immediately: true,
            update_level: false,
        }
    }

    #[test]
    fn wire_bytes_match_schema_v1_field_order() {
        let original = event();
        let bytes = to_bytes(&original).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_u32.to_le_bytes());
        for value in [&original.player_id, &original.dimension_id] {
            expected.extend_from_slice(&(value.len() as u32).to_le_bytes());
            expected.extend_from_slice(value.as_bytes());
        }
        expected.push(1);
        expected.push(0);
        assert_eq!(bytes, expected);
        assert!(bytes.len() <= MAX_PAYLOAD_BYTES);
    }

    #[test]
    fn stable_round_trip_preserves_detached_wake_up_flags() {
        let original = PlayerWakeUp {
            wake_immediately: false,
            update_level: true,
            ..event()
        };
        let bytes = to_bytes(&original).unwrap();
        assert_eq!(
            u32::from_le_bytes(bytes[0..4].try_into().unwrap()),
            1,
            "the schema version must be the first little-endian word"
        );
        assert_eq!(bytes[bytes.len() - 2], 0);
        assert_eq!(bytes[bytes.len() - 1], 1);
        assert_eq!(from_bytes::<PlayerWakeUp>(&bytes).unwrap(), original);
    }

    #[test]
    fn codec_rejects_unsupported_schema_invalid_boolean_and_trailing_bytes() {
        let bytes = to_bytes(&event()).unwrap();

        let mut bad_schema = bytes.clone();
        bad_schema[0..4].copy_from_slice(&2_u32.to_le_bytes());
        assert_eq!(
            from_bytes::<PlayerWakeUp>(&bad_schema).unwrap_err().code(),
            ErrorCode::IncompatibleAbiVersion
        );

        let mut bad_boolean = bytes.clone();
        let last = bad_boolean.len() - 1;
        bad_boolean[last] = 2;
        assert_eq!(
            from_bytes::<PlayerWakeUp>(&bad_boolean).unwrap_err().code(),
            ErrorCode::InvalidBoolean
        );

        let mut trailing = bytes;
        trailing.push(0);
        assert!(from_bytes::<PlayerWakeUp>(&trailing).is_err());
    }

    #[test]
    fn published_limits_match_the_host_contract() {
        assert_eq!("player.wake_up", EVENT_NAME);
        assert_eq!("sjml_v1_event_player.wake_up", EXPORT_NAME);
        assert_eq!(256, MAX_STRING_BYTES);
        assert_eq!(1_024, MAX_PAYLOAD_BYTES);
        assert_eq!(64, MAX_CALLBACKS_PER_EVENT);
    }
}
