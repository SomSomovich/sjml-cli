//! Detached payload and response for cancellable logical-server natural spawn-candidate lists.
//!
//! Register a `world.potential_spawns_attempt` callback with `events.subscribe`
//! and `events.cancel`. Returning `Ok(true)` cancels NeoForge's
//! `LevelEvent.PotentialSpawns`, so the natural spawner builds an empty
//! candidate list and places no entity at that position. The callback is
//! synchronous because the veto is read before the spawner chooses a mob.

use sjml::{StableDecode, StableEncode};

/// Event name used with [`crate::events::subscribe`].
pub const EVENT_NAME: &str = "world.potential_spawns_attempt";
/// Generated ABI export name for the typed spawn-candidate callback.
pub const EXPORT_NAME: &str = "sjml_v1_event_world.potential_spawns_attempt";
/// Maximum UTF-8 byte length of the dimension ID.
pub const MAX_STRING_BYTES: usize = 256;
/// Maximum encoded bytes for one detached spawn-candidate event.
pub const MAX_PAYLOAD_BYTES: usize = 768;
/// Maximum guest callbacks invoked for one spawn-candidate event.
pub const MAX_CALLBACKS_PER_EVENT: usize = 64;
/// Maximum number of candidate spawner entries the host reports.
pub const MAX_CANDIDATES: i32 = 4_096;

/// Synchronous cancellable snapshot captured while the natural spawner builds
/// its candidate list at a position.
///
/// Returning `Ok(true)` from the handler calls `setCanceled(true)` on the
/// NeoForge event, which makes the candidate list empty, so no natural spawn
/// occurs at that position. Returning `Ok(false)` keeps vanilla behavior. The
/// detached snapshot contains the dimension, the lowercase `MobCategory` name,
/// the block position, and the number of candidate spawner entries.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct WorldPotentialSpawnsAttempt {
    /// Resource location of the dimension containing the spawn position.
    pub dimension_id: String,
    /// Lowercase `MobCategory` name, such as `monster` or `creature`.
    pub category: String,
    /// Spawn position x coordinate.
    pub x: i32,
    /// Spawn position y coordinate.
    pub y: i32,
    /// Spawn position z coordinate.
    pub z: i32,
    /// Number of candidate spawner entries, within `0..=4096`.
    pub candidate_count: i32,
}

#[cfg(test)]
mod tests {
    use super::{
        WorldPotentialSpawnsAttempt, EVENT_NAME, EXPORT_NAME, MAX_CALLBACKS_PER_EVENT,
        MAX_CANDIDATES, MAX_PAYLOAD_BYTES, MAX_STRING_BYTES,
    };
    use sjml::{from_bytes, to_bytes, ErrorCode};

    fn event() -> WorldPotentialSpawnsAttempt {
        WorldPotentialSpawnsAttempt {
            dimension_id: "minecraft:overworld".into(),
            category: "monster".into(),
            x: 1,
            y: 64,
            z: -3,
            candidate_count: 5,
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
        append_test_string(&mut expected, &original.category);
        expected.extend_from_slice(&original.x.to_le_bytes());
        expected.extend_from_slice(&original.y.to_le_bytes());
        expected.extend_from_slice(&original.z.to_le_bytes());
        expected.extend_from_slice(&original.candidate_count.to_le_bytes());
        assert_eq!(bytes, expected);
        assert!(bytes.len() <= MAX_PAYLOAD_BYTES);
    }

    #[test]
    fn stable_round_trip_preserves_bounded_candidates() {
        for event in [
            event(),
            WorldPotentialSpawnsAttempt {
                x: -30_000_000,
                y: 320,
                z: 30_000_000,
                candidate_count: 0,
                ..event()
            },
            WorldPotentialSpawnsAttempt {
                dimension_id: "minecraft:the_nether".into(),
                category: "creature".into(),
                candidate_count: MAX_CANDIDATES,
                ..event()
            },
        ] {
            let bytes = to_bytes(&event).unwrap();
            assert_eq!(
                from_bytes::<WorldPotentialSpawnsAttempt>(&bytes).unwrap(),
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
            from_bytes::<WorldPotentialSpawnsAttempt>(&bad_schema)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );

        let mut trailing = bytes;
        trailing.push(0);
        assert!(from_bytes::<WorldPotentialSpawnsAttempt>(&trailing).is_err());
    }

    #[test]
    fn published_limits_match_the_host_contract() {
        assert_eq!("world.potential_spawns_attempt", EVENT_NAME);
        assert_eq!("sjml_v1_event_world.potential_spawns_attempt", EXPORT_NAME);
        assert_eq!(256, MAX_STRING_BYTES);
        assert_eq!(768, MAX_PAYLOAD_BYTES);
        assert_eq!(64, MAX_CALLBACKS_PER_EVENT);
        assert_eq!(4_096, MAX_CANDIDATES);
    }
}
