//! Detached payload and response for cancellable logical-server mob despawns.
//!
//! Register an `entity.despawn_attempt` callback with `events.subscribe` and
//! `events.cancel`. Returning `Ok(true)` sets NeoForge's despawn result to
//! `DENY`, which keeps the mob alive instead of removing it. The callback is
//! synchronous because the result is read as soon as the event returns.

use crate::{StableDecode, StableEncode};

/// Event name for natural mob despawn attempts on the logical server.
pub const ENTITY_DESPAWN_ATTEMPT_EVENT_NAME: &str = "entity.despawn_attempt";
/// Cancellation grant required to veto mob despawn attempts; `events.subscribe` is also required.
pub const ENTITY_DESPAWN_ATTEMPT_CANCEL_CAPABILITY: &str = "events.cancel";
/// Maximum UTF-8 byte length for an entity, type, or dimension identifier.
pub const MAX_ENTITY_DESPAWN_ATTEMPT_ID_BYTES: usize = 256;
/// Maximum world coordinate accepted in one despawn position.
pub const MAX_ENTITY_DESPAWN_ATTEMPT_COORDINATE: f64 = 30_000_000.0;
/// Maximum serialized payload size for one despawn-attempt callback.
pub const MAX_ENTITY_DESPAWN_ATTEMPT_PAYLOAD_BYTES: usize = 1_024;
/// Maximum callbacks invoked for one mob despawn attempt.
pub const MAX_ENTITY_DESPAWN_ATTEMPT_CALLBACKS: usize = 64;

/// Synchronous cancellable snapshot captured when a mob attempts to despawn.
///
/// Returning `Ok(true)` from the handler forbids the natural despawn: the mob
/// stays in the level with no removal, drops, or experience. Returning
/// `Ok(false)` keeps vanilla despawn logic. The detached snapshot contains the
/// mob UUID and type, the dimension, and its bounded world position.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityDespawnAttempt {
    /// Canonical UUID of the mob attempting to despawn.
    pub entity_id: String,
    /// Registry ID of the mob type, such as `minecraft:zombie`.
    pub entity_type_id: String,
    /// Resource location of the dimension containing the mob.
    pub dimension_id: String,
    /// World-space X coordinate of the despawn attempt.
    pub x: f64,
    /// World-space Y coordinate of the despawn attempt.
    pub y: f64,
    /// World-space Z coordinate of the despawn attempt.
    pub z: f64,
}

#[cfg(test)]
mod tests {
    use super::EntityDespawnAttempt;
    use crate::{from_bytes, to_bytes, ErrorCode};

    fn sample() -> EntityDespawnAttempt {
        EntityDespawnAttempt {
            entity_id: "00000000-0000-0000-0000-000000000001".into(),
            entity_type_id: "minecraft:zombie".into(),
            dimension_id: "minecraft:overworld".into(),
            x: 1.5,
            y: 64.0,
            z: -3.25,
        }
    }

    #[test]
    fn despawn_attempt_uses_the_stable_schema_v1_wire_layout() {
        let event = sample();
        let encoded = to_bytes(&event).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_u32.to_le_bytes());
        for value in [&event.entity_id, &event.entity_type_id, &event.dimension_id] {
            expected.extend_from_slice(&(value.len() as u32).to_le_bytes());
            expected.extend_from_slice(value.as_bytes());
        }
        for value in [event.x, event.y, event.z] {
            expected.extend_from_slice(&value.to_bits().to_le_bytes());
        }
        assert_eq!(encoded, expected);
        assert_eq!(from_bytes::<EntityDespawnAttempt>(&encoded).unwrap(), event);
    }

    #[test]
    fn despawn_attempt_round_trips_negative_positions() {
        let mut event = sample();
        event.x = -29_999_999.5;
        event.y = -64.0;
        event.z = 12_345.678;
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(from_bytes::<EntityDespawnAttempt>(&encoded).unwrap(), event);
    }

    #[test]
    fn despawn_attempt_rejects_other_schema_versions() {
        let mut encoded = to_bytes(&sample()).unwrap();
        encoded[..4].copy_from_slice(&2_u32.to_le_bytes());
        assert_eq!(
            from_bytes::<EntityDespawnAttempt>(&encoded)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );
    }

    #[test]
    fn despawn_attempt_constants_match_the_host_contract() {
        assert_eq!(
            super::ENTITY_DESPAWN_ATTEMPT_EVENT_NAME,
            "entity.despawn_attempt"
        );
        assert_eq!(
            super::ENTITY_DESPAWN_ATTEMPT_CANCEL_CAPABILITY,
            "events.cancel"
        );
        assert_eq!(super::MAX_ENTITY_DESPAWN_ATTEMPT_ID_BYTES, 256);
        assert_eq!(super::MAX_ENTITY_DESPAWN_ATTEMPT_PAYLOAD_BYTES, 1_024);
        assert_eq!(super::MAX_ENTITY_DESPAWN_ATTEMPT_CALLBACKS, 64);
        assert_eq!(super::MAX_ENTITY_DESPAWN_ATTEMPT_COORDINATE, 30_000_000.0);
    }
}
