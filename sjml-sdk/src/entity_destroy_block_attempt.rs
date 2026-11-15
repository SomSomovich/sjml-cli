//! Detached payload and veto response for logical-server living-entity block destruction.
//!
//! Register an `entity.destroy_block_attempt` callback with `events.subscribe` and
//! `events.cancel`. Returning `Ok(true)` cancels NeoForge's
//! `LivingDestroyBlockEvent`, so the block survives the attempt. The callback is
//! synchronous because the cancellation must apply before vanilla destroys the block.

use crate::{StableDecode, StableEncode};

/// Event name for living-entity block destruction attempts on the logical server.
pub const ENTITY_DESTROY_BLOCK_ATTEMPT_EVENT_NAME: &str = "entity.destroy_block_attempt";
/// Guest export name the host resolves for one block destruction callback.
pub const ENTITY_DESTROY_BLOCK_ATTEMPT_EXPORT_NAME: &str =
    "sjml_v1_event_entity.destroy_block_attempt";
/// Cancellation grant required to veto block destruction; `events.subscribe` is also required.
pub const ENTITY_DESTROY_BLOCK_ATTEMPT_CANCEL_CAPABILITY: &str = "events.cancel";
/// Schema version of the detached block destruction payload.
pub const ENTITY_DESTROY_BLOCK_ATTEMPT_SCHEMA_VERSION: u32 = 1;
/// Maximum UTF-8 byte length of an entity UUID, type ID, dimension ID, or block ID.
pub const MAX_ENTITY_DESTROY_BLOCK_ATTEMPT_STRING_BYTES: usize = 256;
/// Maximum UTF-8 byte length of the canonical target block state text.
pub const MAX_ENTITY_DESTROY_BLOCK_ATTEMPT_STATE_BYTES: usize = 512;
/// Maximum serialized payload size for one block destruction callback.
pub const MAX_ENTITY_DESTROY_BLOCK_ATTEMPT_PAYLOAD_BYTES: usize = 2_048;
/// Maximum callbacks invoked for one living-entity block destruction attempt.
pub const MAX_ENTITY_DESTROY_BLOCK_ATTEMPT_CALLBACKS: usize = 64;
/// Maximum absolute value accepted for any block coordinate.
pub const MAX_ENTITY_DESTROY_BLOCK_ATTEMPT_COORDINATE: i32 = 30_000_000;

/// Synchronous cancellable snapshot captured when a living entity attempts to break a block.
///
/// NeoForge fires `entity.destroy_block_attempt` through `LivingDestroyBlockEvent`
/// when an ender dragon or wither attempts to destroy a block and when a zombie
/// attempts to break a door. Returning `Ok(true)` cancels the event: the block is
/// not destroyed and no vanilla break behavior runs. Returning `Ok(false)` keeps
/// the vanilla result. The detached snapshot contains the entity UUID and type,
/// the dimension, the target block position, and the canonical block ID and block
/// state text. Both `events.subscribe` and `events.cancel` are required, and the
/// callback must remain synchronous.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityDestroyBlockAttempt {
    /// Canonical UUID of the living entity attempting the break.
    pub entity_id: String,
    /// Registry ID of the entity type, such as `minecraft:zombie`.
    pub entity_type_id: String,
    /// Resource location of the dimension containing the entity.
    pub dimension_id: String,
    /// Block X coordinate of the target.
    pub x: i32,
    /// Block Y coordinate of the target.
    pub y: i32,
    /// Block Z coordinate of the target.
    pub z: i32,
    /// Canonical registry ID of the target block.
    pub block_id: String,
    /// Canonical block state text, bounded by the host to 512 UTF-8 bytes.
    pub block_state: String,
}

#[cfg(test)]
mod tests {
    use super::EntityDestroyBlockAttempt;
    use crate::{from_bytes, to_bytes, ErrorCode};

    fn sample() -> EntityDestroyBlockAttempt {
        EntityDestroyBlockAttempt {
            entity_id: "00000000-0000-0000-0000-000000000002".into(),
            entity_type_id: "minecraft:zombie".into(),
            dimension_id: "minecraft:overworld".into(),
            x: 1,
            y: 64,
            z: -3,
            block_id: "minecraft:oak_door".into(),
            block_state: "minecraft:oak_door[facing=north,half=lower]".into(),
        }
    }

    fn push_string(bytes: &mut Vec<u8>, value: &str) {
        bytes.extend_from_slice(&(value.len() as u32).to_le_bytes());
        bytes.extend_from_slice(value.as_bytes());
    }

    #[test]
    fn encodes_the_java_schema_v1_wire_bytes() {
        let event = sample();
        let mut expected = 1_u32.to_le_bytes().to_vec();
        push_string(&mut expected, &event.entity_id);
        push_string(&mut expected, &event.entity_type_id);
        push_string(&mut expected, &event.dimension_id);
        expected.extend_from_slice(&event.x.to_le_bytes());
        expected.extend_from_slice(&event.y.to_le_bytes());
        expected.extend_from_slice(&event.z.to_le_bytes());
        push_string(&mut expected, &event.block_id);
        push_string(&mut expected, &event.block_state);
        assert_eq!(to_bytes(&event).unwrap(), expected);
        assert_eq!(
            from_bytes::<EntityDestroyBlockAttempt>(&expected).unwrap(),
            event
        );
    }

    #[test]
    fn destroy_block_attempt_rejects_other_schema_versions() {
        let mut encoded = to_bytes(&sample()).unwrap();
        encoded[..4].copy_from_slice(&2_u32.to_le_bytes());
        assert_eq!(
            from_bytes::<EntityDestroyBlockAttempt>(&encoded)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );
    }

    #[test]
    fn destroy_block_attempt_constants_match_the_host_contract() {
        assert_eq!(
            super::ENTITY_DESTROY_BLOCK_ATTEMPT_EVENT_NAME,
            "entity.destroy_block_attempt"
        );
        assert_eq!(
            super::ENTITY_DESTROY_BLOCK_ATTEMPT_EXPORT_NAME,
            "sjml_v1_event_entity.destroy_block_attempt"
        );
        assert_eq!(
            super::ENTITY_DESTROY_BLOCK_ATTEMPT_CANCEL_CAPABILITY,
            "events.cancel"
        );
        assert_eq!(super::ENTITY_DESTROY_BLOCK_ATTEMPT_SCHEMA_VERSION, 1);
        assert_eq!(super::MAX_ENTITY_DESTROY_BLOCK_ATTEMPT_STRING_BYTES, 256);
        assert_eq!(super::MAX_ENTITY_DESTROY_BLOCK_ATTEMPT_STATE_BYTES, 512);
        assert_eq!(super::MAX_ENTITY_DESTROY_BLOCK_ATTEMPT_PAYLOAD_BYTES, 2_048);
        assert_eq!(super::MAX_ENTITY_DESTROY_BLOCK_ATTEMPT_CALLBACKS, 64);
        assert_eq!(
            super::MAX_ENTITY_DESTROY_BLOCK_ATTEMPT_COORDINATE,
            30_000_000
        );
    }
}
