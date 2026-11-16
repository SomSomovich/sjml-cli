//! Detached payload for cancellable logical-server spawn placement attempts.
//!
//! Register an `entity.spawn_placement_attempt` callback with `events.subscribe`
//! and `events.cancel`. Return `Ok(true)` to make the host deny the evaluated
//! placement position, and keep the callback synchronous because the vanilla
//! check reads the result as soon as the event returns.

use sjml::{StableDecode, StableEncode};

/// Event name emitted while NeoForge evaluates a natural spawn placement.
pub const ENTITY_SPAWN_PLACEMENT_ATTEMPT_EVENT_NAME: &str = "entity.spawn_placement_attempt";
/// Guest export the host invokes for one `entity.spawn_placement_attempt` callback.
pub const ENTITY_SPAWN_PLACEMENT_ATTEMPT_EXPORT_NAME: &str =
    "sjml_v1_event_entity.spawn_placement_attempt";
/// Maximum UTF-8 byte length of an entity type, dimension, or spawn type.
pub const MAX_ENTITY_SPAWN_PLACEMENT_ATTEMPT_STRING_BYTES: usize = 256;
/// Maximum callbacks the host invokes for one spawn placement attempt.
pub const MAX_ENTITY_SPAWN_PLACEMENT_ATTEMPT_CALLBACKS: usize = 64;
/// Maximum encoded payload size for one detached spawn placement callback.
pub const MAX_ENTITY_SPAWN_PLACEMENT_ATTEMPT_PAYLOAD_BYTES: usize = 768;

/// Synchronous cancellable snapshot captured during a spawn placement check.
///
/// Return `Ok(true)` from the callback to call `setResult(Result.DENY)` on the
/// NeoForge `MobSpawnEvent.SpawnPlacementCheck`, which rejects the placement.
/// All values are detached; no entity, level, or host handle crosses the guest
/// ABI. Events whose result is already non-default are never dispatched.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntitySpawnPlacementAttempt {
    /// Canonical registry resource location of the entity type.
    pub entity_type_id: String,
    /// Resource location of the dimension evaluating the placement.
    pub dimension_id: String,
    /// Block x coordinate of the evaluated position.
    pub x: i32,
    /// Block y coordinate of the evaluated position.
    pub y: i32,
    /// Block z coordinate of the evaluated position.
    pub z: i32,
    /// Lowercase `MobSpawnType` name, such as `natural` or `spawn_egg`.
    pub spawn_type: String,
    /// Vanilla spawn placement result before guest handlers run.
    pub default_result: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sjml::{from_bytes, to_bytes};

    fn event() -> EntitySpawnPlacementAttempt {
        EntitySpawnPlacementAttempt {
            entity_type_id: "minecraft:zombie".into(),
            dimension_id: "minecraft:overworld".into(),
            x: 10,
            y: 64,
            z: -5,
            spawn_type: "natural".into(),
            default_result: true,
        }
    }

    #[test]
    fn event_round_trips_the_java_wire_layout() {
        let event = event();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_string(&mut expected, &event.entity_type_id);
        append_string(&mut expected, &event.dimension_id);
        expected.extend_from_slice(&event.x.to_le_bytes());
        expected.extend_from_slice(&event.y.to_le_bytes());
        expected.extend_from_slice(&event.z.to_le_bytes());
        append_string(&mut expected, &event.spawn_type);
        expected.push(1);

        assert_eq!(to_bytes(&event).unwrap(), expected);
        assert_eq!(
            from_bytes::<EntitySpawnPlacementAttempt>(&expected).unwrap(),
            event
        );
    }

    #[test]
    fn false_default_result_encodes_as_a_zero_byte() {
        let event = EntitySpawnPlacementAttempt {
            default_result: false,
            ..event()
        };
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(encoded.last().copied(), Some(0));
        assert_eq!(
            from_bytes::<EntitySpawnPlacementAttempt>(&encoded).unwrap(),
            event
        );
    }

    #[test]
    fn unsupported_schema_version_is_rejected() {
        let mut encoded = to_bytes(&event()).unwrap();
        encoded[..4].copy_from_slice(&2_i32.to_le_bytes());
        assert_eq!(
            from_bytes::<EntitySpawnPlacementAttempt>(&encoded)
                .unwrap_err()
                .code(),
            sjml::ErrorCode::IncompatibleAbiVersion
        );
    }

    fn append_string(output: &mut Vec<u8>, value: &str) {
        output.extend_from_slice(&(value.len() as i32).to_le_bytes());
        output.extend_from_slice(value.as_bytes());
    }
}
