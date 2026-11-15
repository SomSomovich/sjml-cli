//! Detached payload for cancellable logical-server entity finalize-spawn attempts.
//!
//! Register an `entity.finalize_spawn_attempt` callback with `events.subscribe`
//! and `events.cancel`. Return `Ok(true)` to prevent the mob from spawning, and
//! keep the callback synchronous because NeoForge finalizes the entity
//! immediately after the event returns.

use sjml::{StableDecode, StableEncode};

/// Event name emitted before NeoForge finalizes a mob's initial spawn state.
pub const ENTITY_FINALIZE_SPAWN_ATTEMPT_EVENT_NAME: &str = "entity.finalize_spawn_attempt";
/// Guest export the host invokes for one `entity.finalize_spawn_attempt` callback.
pub const ENTITY_FINALIZE_SPAWN_ATTEMPT_EXPORT_NAME: &str =
    "sjml_v1_event_entity.finalize_spawn_attempt";
/// Maximum UTF-8 byte length of an entity UUID, entity type, or dimension.
pub const MAX_ENTITY_FINALIZE_SPAWN_ATTEMPT_STRING_BYTES: usize = 256;
/// Maximum absolute value accepted for each entity coordinate.
pub const MAX_ENTITY_FINALIZE_SPAWN_ATTEMPT_COORDINATE: f64 = 30_000_000.0;
/// Maximum effective difficulty accepted by the payload schema.
pub const MAX_ENTITY_FINALIZE_SPAWN_ATTEMPT_DIFFICULTY: f32 = 1_000_000.0;
/// Maximum callbacks the host invokes for one finalize-spawn attempt.
pub const MAX_ENTITY_FINALIZE_SPAWN_ATTEMPT_CALLBACKS: usize = 64;
/// Maximum encoded payload size for one detached finalize-spawn callback.
pub const MAX_ENTITY_FINALIZE_SPAWN_ATTEMPT_PAYLOAD_BYTES: usize = 2_048;

/// Synchronous cancellable snapshot captured before a mob is added to a level.
///
/// Return `Ok(true)` from the callback to call `setSpawnCancelled(true)` on the
/// NeoForge event, which blocks the spawn entirely. All values are detached; no
/// `Mob`, level, or host handle crosses the guest ABI.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityFinalizeSpawnAttempt {
    /// Canonical lowercase UUID of the mob about to be spawned.
    pub entity_id: String,
    /// Canonical registry resource location of the entity type.
    pub entity_type_id: String,
    /// Resource location of the dimension receiving the mob.
    pub dimension_id: String,
    /// Spawn X coordinate, finite as an IEEE-754 `f64`.
    pub x: f64,
    /// Spawn Y coordinate, finite as an IEEE-754 `f64`.
    pub y: f64,
    /// Spawn Z coordinate, finite as an IEEE-754 `f64`.
    pub z: f64,
    /// Lowercase `MobSpawnType` name, such as `natural` or `spawn_egg`.
    pub spawn_type: String,
    /// Effective local difficulty scaled to the `0.0..=1_000_000.0` range.
    pub difficulty: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sjml::{from_bytes, to_bytes};

    fn event() -> EntityFinalizeSpawnAttempt {
        EntityFinalizeSpawnAttempt {
            entity_id: "00000000-0000-0000-0000-000000000001".into(),
            entity_type_id: "minecraft:zombie".into(),
            dimension_id: "minecraft:overworld".into(),
            x: 1.5,
            y: 64.0,
            z: -2.25,
            spawn_type: "natural".into(),
            difficulty: 2.5,
        }
    }

    #[test]
    fn event_round_trips_the_java_wire_layout() {
        let event = event();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_string(&mut expected, &event.entity_id);
        append_string(&mut expected, &event.entity_type_id);
        append_string(&mut expected, &event.dimension_id);
        expected.extend_from_slice(&event.x.to_bits().to_le_bytes());
        expected.extend_from_slice(&event.y.to_bits().to_le_bytes());
        expected.extend_from_slice(&event.z.to_bits().to_le_bytes());
        append_string(&mut expected, &event.spawn_type);
        expected.extend_from_slice(&event.difficulty.to_bits().to_le_bytes());

        assert_eq!(to_bytes(&event).unwrap(), expected);
        assert_eq!(
            from_bytes::<EntityFinalizeSpawnAttempt>(&expected).unwrap(),
            event
        );
    }

    #[test]
    fn unsupported_schema_version_is_rejected() {
        let mut encoded = to_bytes(&event()).unwrap();
        encoded[..4].copy_from_slice(&2_i32.to_le_bytes());
        assert_eq!(
            from_bytes::<EntityFinalizeSpawnAttempt>(&encoded)
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
