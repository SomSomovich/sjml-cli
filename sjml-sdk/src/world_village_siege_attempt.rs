//! Detached payload and response for cancellable logical-server village siege attempts.
//!
//! Register a `world.village_siege_attempt` callback with `events.subscribe` and
//! `events.cancel`. Returning `Ok(true)` cancels NeoForge's `VillageSiegeEvent`,
//! so the zombie siege never starts. The callback is synchronous because the
//! veto is read before the siege searches for a spawn position.

use sjml::{StableDecode, StableEncode};

/// Event name used with [`crate::events::subscribe`].
pub const EVENT_NAME: &str = "world.village_siege_attempt";
/// Generated ABI export name for the typed village siege callback.
pub const EXPORT_NAME: &str = "sjml_v1_event_world.village_siege_attempt";
/// Maximum UTF-8 byte length of the dimension ID or player UUID.
pub const MAX_STRING_BYTES: usize = 256;
/// Maximum encoded bytes for one detached village siege event.
pub const MAX_PAYLOAD_BYTES: usize = 1_024;
/// Maximum guest callbacks invoked for one village siege event.
pub const MAX_CALLBACKS_PER_EVENT: usize = 64;
/// Maximum absolute coordinate accepted for the attempted siege spawn position.
pub const MAX_COORDINATE: f64 = 30_000_000.0;

/// Synchronous cancellable snapshot captured when a village siege would start.
///
/// Returning `Ok(true)` from the handler cancels the siege attempt, so no
/// zombies are placed for the targeted village. Returning `Ok(false)` keeps
/// vanilla behavior. The detached snapshot contains the dimension, the target
/// player UUID, and the attempted spawn position.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct WorldVillageSiegeAttempt {
    /// Resource location of the dimension where the siege would start.
    pub dimension_id: String,
    /// Canonical UUID of the player the siege targets.
    pub player_id: String,
    /// World-space X coordinate of the attempted siege spawn.
    pub x: f64,
    /// World-space Y coordinate of the attempted siege spawn.
    pub y: f64,
    /// World-space Z coordinate of the attempted siege spawn.
    pub z: f64,
}

#[cfg(test)]
mod tests {
    use super::{
        WorldVillageSiegeAttempt, EVENT_NAME, EXPORT_NAME, MAX_CALLBACKS_PER_EVENT, MAX_COORDINATE,
        MAX_PAYLOAD_BYTES, MAX_STRING_BYTES,
    };
    use sjml::{from_bytes, to_bytes, ErrorCode};

    fn event() -> WorldVillageSiegeAttempt {
        WorldVillageSiegeAttempt {
            dimension_id: "minecraft:overworld".into(),
            player_id: "00000000-0000-0000-0000-000000000001".into(),
            x: 1.5,
            y: 64.0,
            z: -3.25,
        }
    }

    fn append_test_string(output: &mut Vec<u8>, value: &str) {
        output.extend_from_slice(&(value.len() as i32).to_le_bytes());
        output.extend_from_slice(value.as_bytes());
    }

    #[test]
    fn wire_bytes_match_schema_v1_field_order() {
        let original = event();
        let bytes = to_bytes(&original).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_i32.to_le_bytes());
        append_test_string(&mut expected, &original.dimension_id);
        append_test_string(&mut expected, &original.player_id);
        expected.extend_from_slice(&original.x.to_le_bytes());
        expected.extend_from_slice(&original.y.to_le_bytes());
        expected.extend_from_slice(&original.z.to_le_bytes());
        assert_eq!(bytes, expected);
        assert!(bytes.len() <= MAX_PAYLOAD_BYTES);
    }

    #[test]
    fn stable_round_trip_preserves_bounded_coordinates() {
        for event in [
            event(),
            WorldVillageSiegeAttempt {
                x: -MAX_COORDINATE,
                y: MAX_COORDINATE,
                z: 0.0,
                ..event()
            },
            WorldVillageSiegeAttempt {
                dimension_id: "minecraft:the_end".into(),
                ..event()
            },
        ] {
            let bytes = to_bytes(&event).unwrap();
            assert_eq!(
                from_bytes::<WorldVillageSiegeAttempt>(&bytes).unwrap(),
                event
            );
        }
    }

    #[test]
    fn codec_rejects_unsupported_schema_and_trailing_bytes() {
        let bytes = to_bytes(&event()).unwrap();

        let mut bad_schema = bytes.clone();
        bad_schema[0..4].copy_from_slice(&2_i32.to_le_bytes());
        assert_eq!(
            from_bytes::<WorldVillageSiegeAttempt>(&bad_schema)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );

        let mut trailing = bytes;
        trailing.push(0);
        assert!(from_bytes::<WorldVillageSiegeAttempt>(&trailing).is_err());
    }

    #[test]
    fn published_limits_match_the_host_contract() {
        assert_eq!("world.village_siege_attempt", EVENT_NAME);
        assert_eq!("sjml_v1_event_world.village_siege_attempt", EXPORT_NAME);
        assert_eq!(256, MAX_STRING_BYTES);
        assert_eq!(1_024, MAX_PAYLOAD_BYTES);
        assert_eq!(64, MAX_CALLBACKS_PER_EVENT);
        assert_eq!(30_000_000.0, MAX_COORDINATE);
    }
}
