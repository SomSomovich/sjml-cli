//! Detached observation of a logical-server living entity jumping.
//!
//! Subscribe with `events::subscribe("entity.jump", EventPriority::Normal)` and a typed
//! `#[event(name = "entity.jump")]` callback returning `SjmlResult<()>`. The notification is
//! observational: it cannot cancel or change the jump, and the host does not write any field
//! back to the entity. Callbacks run on the owning server thread, and the host invokes at most
//! [`MAX_CALLBACKS`] callbacks per jump.

use sjml::{StableDecode, StableEncode};

/// Event name used with [`crate::events::subscribe`].
pub const EVENT_NAME: &str = "entity.jump";
/// Generated ABI export name for the typed jump callback.
pub const EXPORT_NAME: &str = "sjml_v1_event_entity.jump";
/// Schema version of the detached jump payload.
pub const SCHEMA_VERSION: u32 = 1;
/// Maximum UTF-8 byte length of the entity UUID, entity type ID, and dimension ID.
pub const MAX_STRING_BYTES: usize = 256;
/// Maximum encoded bytes for one detached jump event.
pub const MAX_PAYLOAD_BYTES: usize = 512;
/// Maximum guest callbacks invoked for one jump event.
pub const MAX_CALLBACKS: usize = 64;

/// Detached jump notification. It contains no `LivingEntity`, level, or host handle.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityJump {
    /// Canonical lowercase UUID of the entity that jumped.
    pub entity_id: String,
    /// Registry ID of the entity type, such as `minecraft:zombie`.
    pub entity_type_id: String,
    /// Resource ID of the dimension containing the entity.
    pub dimension_id: String,
}

#[cfg(test)]
mod tests {
    use super::{
        EntityJump, EVENT_NAME, EXPORT_NAME, MAX_CALLBACKS, MAX_PAYLOAD_BYTES, MAX_STRING_BYTES,
        SCHEMA_VERSION,
    };
    use sjml::{from_bytes, to_bytes, ErrorCode};

    fn event() -> EntityJump {
        EntityJump {
            entity_id: "68ccfd1a-588d-4eb4-a0c9-31046942d74f".into(),
            entity_type_id: "minecraft:zombie".into(),
            dimension_id: "minecraft:overworld".into(),
        }
    }

    fn push_string(bytes: &mut Vec<u8>, value: &str) {
        bytes.extend_from_slice(&(value.len() as u32).to_le_bytes());
        bytes.extend_from_slice(value.as_bytes());
    }

    #[test]
    fn wire_bytes_match_schema_v1_field_order() {
        let original = event();
        let bytes = to_bytes(&original).unwrap();
        let mut expected = SCHEMA_VERSION.to_le_bytes().to_vec();
        push_string(&mut expected, &original.entity_id);
        push_string(&mut expected, &original.entity_type_id);
        push_string(&mut expected, &original.dimension_id);
        assert_eq!(bytes, expected);
        assert!(bytes.len() <= MAX_PAYLOAD_BYTES);
    }

    #[test]
    fn stable_round_trip_preserves_detached_identity() {
        let original = EntityJump {
            dimension_id: "minecraft:the_nether".into(),
            ..event()
        };
        let bytes = to_bytes(&original).unwrap();
        assert_eq!(
            u32::from_le_bytes(bytes[0..4].try_into().unwrap()),
            SCHEMA_VERSION
        );
        assert_eq!(from_bytes::<EntityJump>(&bytes).unwrap(), original);
    }

    #[test]
    fn codec_rejects_unsupported_schema_and_trailing_bytes() {
        let bytes = to_bytes(&event()).unwrap();

        let mut bad_schema = bytes.clone();
        bad_schema[0..4].copy_from_slice(&2_u32.to_le_bytes());
        assert_eq!(
            from_bytes::<EntityJump>(&bad_schema).unwrap_err().code(),
            ErrorCode::IncompatibleAbiVersion
        );

        let mut trailing = bytes;
        trailing.push(0);
        assert!(from_bytes::<EntityJump>(&trailing).is_err());
    }

    #[test]
    fn published_limits_match_the_host_contract() {
        assert_eq!("entity.jump", EVENT_NAME);
        assert_eq!("sjml_v1_event_entity.jump", EXPORT_NAME);
        assert_eq!(1, SCHEMA_VERSION);
        assert_eq!(256, MAX_STRING_BYTES);
        assert_eq!(512, MAX_PAYLOAD_BYTES);
        assert_eq!(64, MAX_CALLBACKS);
    }
}
