//! Detached observation of a logical-server player changing dimension.
//!
//! Subscribe with `events::subscribe("player.changed_dimension", EventPriority::Normal)` and a
//! typed `#[event(name = "player.changed_dimension")]` callback returning `SjmlResult<()>`. The
//! notification is observational: it cannot cancel or change the completed dimension change. It is
//! delivered on the owning server thread after the player has moved, and the host invokes at most
//! [`MAX_CALLBACKS_PER_EVENT`] callbacks per change. Because no veto is possible, `#[event_async]`
//! handlers are also allowed.

use sjml::{StableDecode, StableEncode};

/// Event name used with [`crate::events::subscribe`].
pub const EVENT_NAME: &str = "player.changed_dimension";
/// Generated ABI export name for the typed dimension-change callback.
pub const EXPORT_NAME: &str = "sjml_v1_event_player.changed_dimension";
/// Maximum UTF-8 byte length of the player UUID and each dimension ID.
pub const MAX_STRING_BYTES: usize = 256;
/// Maximum encoded bytes for one detached dimension-change event.
pub const MAX_PAYLOAD_BYTES: usize = 768;
/// Maximum guest callbacks invoked for one dimension-change event.
pub const MAX_CALLBACKS_PER_EVENT: usize = 64;

/// Detached dimension-change notification. It contains no `Player`, level, or host handle.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerChangedDimension {
    /// Canonical lowercase UUID of the player that changed dimension.
    pub player_id: String,
    /// Resource location of the dimension the player left.
    pub from_dimension_id: String,
    /// Resource location of the dimension the player entered.
    pub to_dimension_id: String,
}

#[cfg(test)]
mod tests {
    use super::{
        PlayerChangedDimension, EVENT_NAME, EXPORT_NAME, MAX_CALLBACKS_PER_EVENT,
        MAX_PAYLOAD_BYTES, MAX_STRING_BYTES,
    };
    use sjml::{from_bytes, to_bytes, ErrorCode};

    fn event() -> PlayerChangedDimension {
        PlayerChangedDimension {
            player_id: "123e4567-e89b-12d3-a456-426614174000".into(),
            from_dimension_id: "minecraft:overworld".into(),
            to_dimension_id: "minecraft:the_nether".into(),
        }
    }

    #[test]
    fn wire_bytes_match_schema_v1_field_order() {
        let original = event();
        let bytes = to_bytes(&original).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_u32.to_le_bytes());
        for value in [
            &original.player_id,
            &original.from_dimension_id,
            &original.to_dimension_id,
        ] {
            expected.extend_from_slice(&(value.len() as u32).to_le_bytes());
            expected.extend_from_slice(value.as_bytes());
        }
        assert_eq!(bytes, expected);
        assert!(bytes.len() <= MAX_PAYLOAD_BYTES);
    }

    #[test]
    fn stable_round_trip_preserves_dimension_direction() {
        let original = PlayerChangedDimension {
            from_dimension_id: "minecraft:the_nether".into(),
            to_dimension_id: "minecraft:overworld".into(),
            ..event()
        };
        let bytes = to_bytes(&original).unwrap();
        assert_eq!(
            u32::from_le_bytes(bytes[0..4].try_into().unwrap()),
            1,
            "the schema version must be the first little-endian word"
        );
        assert_eq!(
            from_bytes::<PlayerChangedDimension>(&bytes).unwrap(),
            original
        );
    }

    #[test]
    fn codec_rejects_unsupported_schema_and_trailing_bytes() {
        let bytes = to_bytes(&event()).unwrap();

        let mut bad_schema = bytes.clone();
        bad_schema[0..4].copy_from_slice(&2_u32.to_le_bytes());
        assert_eq!(
            from_bytes::<PlayerChangedDimension>(&bad_schema)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );

        let mut trailing = bytes;
        trailing.push(0);
        assert!(from_bytes::<PlayerChangedDimension>(&trailing).is_err());
    }

    #[test]
    fn published_limits_match_the_host_contract() {
        assert_eq!("player.changed_dimension", EVENT_NAME);
        assert_eq!("sjml_v1_event_player.changed_dimension", EXPORT_NAME);
        assert_eq!(256, MAX_STRING_BYTES);
        assert_eq!(768, MAX_PAYLOAD_BYTES);
        assert_eq!(64, MAX_CALLBACKS_PER_EVENT);
    }
}
