//! Detached observation of a logical-server flying player fall.
//!
//! Subscribe with `events::subscribe("player.flyable_fall", EventPriority::Normal)` and a typed
//! `#[event(name = "player.flyable_fall")]` callback returning `SjmlResult<()>`. The notification
//! is observational: it cannot cancel or change the fall distance or damage multiplier. Callbacks
//! run on the owning server thread, and the host invokes at most [`MAX_CALLBACKS`] callbacks per
//! fall.

use sjml::{StableDecode, StableEncode};

/// Event name used with [`crate::events::subscribe`].
pub const EVENT_NAME: &str = "player.flyable_fall";
/// Generated ABI export name for the typed flying-player fall callback.
pub const EXPORT_NAME: &str = "sjml_v1_event_player.flyable_fall";
/// Maximum UTF-8 byte length of the player UUID and dimension ID.
pub const MAX_STRING_BYTES: usize = 256;
/// Maximum encoded bytes for one detached flying-player fall event.
pub const MAX_PAYLOAD_BYTES: usize = 768;
/// Maximum guest callbacks invoked for one flying-player fall event.
pub const MAX_CALLBACKS: usize = 64;

/// Detached flying-player fall notification. It contains no `Player`, level, or host handle.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerFlyableFall {
    /// Canonical lowercase UUID of the falling player.
    pub player_id: String,
    /// Resource location of the dimension containing the player.
    pub dimension_id: String,
    /// Current fall distance in blocks, bounded to `0..=10_000`.
    pub distance: f32,
    /// Current fall damage multiplier, bounded to `0..=10_000`.
    pub multiplier: f32,
}

#[cfg(test)]
mod tests {
    use super::{
        PlayerFlyableFall, EVENT_NAME, EXPORT_NAME, MAX_CALLBACKS, MAX_PAYLOAD_BYTES,
        MAX_STRING_BYTES,
    };
    use sjml::{from_bytes, to_bytes, ErrorCode};

    fn event() -> PlayerFlyableFall {
        PlayerFlyableFall {
            player_id: "123e4567-e89b-12d3-a456-426614174000".into(),
            dimension_id: "minecraft:overworld".into(),
            distance: 3.5,
            multiplier: 1.25,
        }
    }

    #[test]
    fn wire_bytes_match_schema_v1_field_order() {
        let original = event();
        let bytes = to_bytes(&original).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_i32.to_le_bytes());
        for value in [&original.player_id, &original.dimension_id] {
            expected.extend_from_slice(&(value.len() as u32).to_le_bytes());
            expected.extend_from_slice(value.as_bytes());
        }
        expected.extend_from_slice(&original.distance.to_le_bytes());
        expected.extend_from_slice(&original.multiplier.to_le_bytes());
        assert_eq!(bytes, expected);
        assert!(bytes.len() <= MAX_PAYLOAD_BYTES);
    }

    #[test]
    fn stable_round_trip_preserves_detached_fall_values() {
        let original = PlayerFlyableFall {
            distance: 12.75,
            multiplier: 2.5,
            ..event()
        };
        let bytes = to_bytes(&original).unwrap();
        assert_eq!(
            i32::from_le_bytes(bytes[0..4].try_into().unwrap()),
            1,
            "the schema version must be the first little-endian word"
        );
        assert_eq!(
            &bytes[bytes.len() - 8..bytes.len() - 4],
            &12.75_f32.to_le_bytes()
        );
        assert_eq!(&bytes[bytes.len() - 4..], &2.5_f32.to_le_bytes());
        assert_eq!(from_bytes::<PlayerFlyableFall>(&bytes).unwrap(), original);
    }

    #[test]
    fn codec_rejects_unsupported_schema_and_trailing_bytes() {
        let bytes = to_bytes(&event()).unwrap();

        let mut bad_schema = bytes.clone();
        bad_schema[0..4].copy_from_slice(&2_i32.to_le_bytes());
        assert_eq!(
            from_bytes::<PlayerFlyableFall>(&bad_schema)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );

        let mut trailing = bytes;
        trailing.push(0);
        assert!(from_bytes::<PlayerFlyableFall>(&trailing).is_err());
    }

    #[test]
    fn published_limits_match_the_host_contract() {
        assert_eq!("player.flyable_fall", EVENT_NAME);
        assert_eq!("sjml_v1_event_player.flyable_fall", EXPORT_NAME);
        assert_eq!(256, MAX_STRING_BYTES);
        assert_eq!(768, MAX_PAYLOAD_BYTES);
        assert_eq!(64, MAX_CALLBACKS);
    }
}
