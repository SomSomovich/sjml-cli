//! Detached observation of the logical-server sleep-finish time decision.
//!
//! Subscribe with `events::subscribe("world.sleep_finished_time", EventPriority::Normal)` and a
//! typed `#[event(name = "world.sleep_finished_time")]` callback returning `SjmlResult<()>`. The
//! notification is observational: it cannot change or cancel the proposed time. The host fires it
//! on the owning server thread after all players finish sleeping, and invokes at most
//! [`MAX_CALLBACKS_PER_EVENT`] callbacks per decision.

use sjml::{StableDecode, StableEncode};

/// Event name used with [`crate::events::subscribe`].
pub const EVENT_NAME: &str = "world.sleep_finished_time";
/// Generated ABI export name for the typed sleep-finish callback.
pub const EXPORT_NAME: &str = "sjml_v1_event_world.sleep_finished_time";
/// Maximum UTF-8 byte length of the dimension ID.
pub const MAX_STRING_BYTES: usize = 256;
/// Maximum encoded bytes for one detached sleep-finish event.
pub const MAX_PAYLOAD_BYTES: usize = 1_024;
/// Maximum guest callbacks invoked for one sleep-finish event.
pub const MAX_CALLBACKS_PER_EVENT: usize = 64;
/// Maximum absolute world time, in ticks, representable in this payload.
pub const MAX_TIME_TICKS: i64 = 1_000_000_000_000;

/// Detached sleep-finish notification. It contains no level, player, or host handle.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct WorldSleepFinishedTime {
    /// Resource location of the dimension whose players finished sleeping.
    pub dimension_id: String,
    /// World time the server proposes to set after all players wake up.
    pub new_time: i64,
    /// Minimum time the server accepts; vanilla rejects lower values.
    pub current_time: i64,
}

#[cfg(test)]
mod tests {
    use super::{
        WorldSleepFinishedTime, EVENT_NAME, EXPORT_NAME, MAX_CALLBACKS_PER_EVENT,
        MAX_PAYLOAD_BYTES, MAX_STRING_BYTES, MAX_TIME_TICKS,
    };
    use sjml::{from_bytes, to_bytes, ErrorCode};

    fn event() -> WorldSleepFinishedTime {
        WorldSleepFinishedTime {
            dimension_id: "minecraft:overworld".into(),
            new_time: 1234,
            current_time: 0,
        }
    }

    #[test]
    fn wire_bytes_match_schema_v1_field_order() {
        let original = event();
        let bytes = to_bytes(&original).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_u32.to_le_bytes());
        expected.extend_from_slice(&(original.dimension_id.len() as u32).to_le_bytes());
        expected.extend_from_slice(original.dimension_id.as_bytes());
        expected.extend_from_slice(&original.new_time.to_le_bytes());
        expected.extend_from_slice(&original.current_time.to_le_bytes());
        assert_eq!(bytes, expected);
        assert!(bytes.len() <= MAX_PAYLOAD_BYTES);
    }

    #[test]
    fn stable_round_trip_preserves_signed_times_and_dimension() {
        for event in [
            event(),
            WorldSleepFinishedTime {
                new_time: -1,
                current_time: -MAX_TIME_TICKS,
                ..event()
            },
            WorldSleepFinishedTime {
                dimension_id: "minecraft:the_end".into(),
                new_time: MAX_TIME_TICKS,
                current_time: 0,
            },
        ] {
            let bytes = to_bytes(&event).unwrap();
            assert_eq!(from_bytes::<WorldSleepFinishedTime>(&bytes).unwrap(), event);
        }
    }

    #[test]
    fn codec_rejects_unsupported_schema_and_trailing_bytes() {
        let bytes = to_bytes(&event()).unwrap();

        let mut bad_schema = bytes.clone();
        bad_schema[0..4].copy_from_slice(&2_u32.to_le_bytes());
        assert_eq!(
            from_bytes::<WorldSleepFinishedTime>(&bad_schema)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );

        let mut trailing = bytes;
        trailing.push(0);
        assert!(from_bytes::<WorldSleepFinishedTime>(&trailing).is_err());
    }

    #[test]
    fn published_limits_match_the_host_contract() {
        assert_eq!("world.sleep_finished_time", EVENT_NAME);
        assert_eq!("sjml_v1_event_world.sleep_finished_time", EXPORT_NAME);
        assert_eq!(256, MAX_STRING_BYTES);
        assert_eq!(1_024, MAX_PAYLOAD_BYTES);
        assert_eq!(64, MAX_CALLBACKS_PER_EVENT);
        assert_eq!(1_000_000_000_000, MAX_TIME_TICKS);
    }
}
