//! Detached observation of a logical-server living entity resolving its air supply.
//!
//! Subscribe with `events::subscribe("entity.breathe", EventPriority::Normal)` and a typed
//! `#[event(name = "entity.breathe")]` callback returning `SjmlResult<()>`. The notification is
//! observational: it cannot cancel the tick, and the host does not write `can_breathe`,
//! `consume_air`, or `refill_air` back to the entity. Callbacks run on the owning logical-server
//! thread, and the host invokes at most [`MAX_CALLBACKS`] callbacks per breath tick.

use sjml::{StableDecode, StableEncode};

/// Event name used with [`crate::events::subscribe`].
pub const EVENT_NAME: &str = "entity.breathe";
/// Generated ABI export name for the typed breath callback.
pub const EXPORT_NAME: &str = "sjml_v1_event_entity.breathe";
/// Schema version of the detached breath payload.
pub const SCHEMA_VERSION: u32 = 1;
/// Maximum UTF-8 byte length of the entity UUID, entity type ID, and dimension ID.
pub const MAX_STRING_BYTES: usize = 256;
/// Minimum accepted value for `consume_air` and `refill_air`.
pub const MIN_AIR_AMOUNT: i32 = 0;
/// Maximum accepted value for `consume_air` and `refill_air`.
pub const MAX_AIR_AMOUNT: i32 = 1_000_000;
/// Maximum encoded bytes for one detached breath event.
pub const MAX_PAYLOAD_BYTES: usize = 768;
/// Maximum guest callbacks invoked for one breath event.
pub const MAX_CALLBACKS: usize = 64;

/// Detached breath notification. It contains no `LivingEntity`, level, or host handle.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityBreathe {
    /// Canonical lowercase UUID of the entity resolving its air supply.
    pub entity_id: String,
    /// Registry ID of the entity type, such as `minecraft:zombie`.
    pub entity_type_id: String,
    /// Resource ID of the dimension containing the entity.
    pub dimension_id: String,
    /// Whether the entity can breathe this tick.
    pub can_breathe: bool,
    /// Air units removed when the entity cannot breathe.
    pub consume_air: i32,
    /// Air units restored when the entity can breathe.
    pub refill_air: i32,
}

#[cfg(test)]
mod tests {
    use super::{
        EntityBreathe, EVENT_NAME, EXPORT_NAME, MAX_AIR_AMOUNT, MAX_CALLBACKS, MAX_PAYLOAD_BYTES,
        MAX_STRING_BYTES, MIN_AIR_AMOUNT, SCHEMA_VERSION,
    };
    use sjml::{from_bytes, to_bytes, ErrorCode};

    fn event() -> EntityBreathe {
        EntityBreathe {
            entity_id: "68ccfd1a-588d-4eb4-a0c9-31046942d74f".into(),
            entity_type_id: "minecraft:zombie".into(),
            dimension_id: "minecraft:overworld".into(),
            can_breathe: false,
            consume_air: 1,
            refill_air: 0,
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
        let mut expected = 1_u32.to_le_bytes().to_vec();
        push_string(&mut expected, &original.entity_id);
        push_string(&mut expected, &original.entity_type_id);
        push_string(&mut expected, &original.dimension_id);
        expected.push(0);
        expected.extend_from_slice(&1_i32.to_le_bytes());
        expected.extend_from_slice(&0_i32.to_le_bytes());
        assert_eq!(bytes, expected);
        assert!(bytes.len() <= MAX_PAYLOAD_BYTES);
    }

    #[test]
    fn stable_round_trip_preserves_detached_air_values() {
        let original = EntityBreathe {
            can_breathe: true,
            consume_air: 30,
            refill_air: 4,
            ..event()
        };
        let bytes = to_bytes(&original).unwrap();
        assert_eq!(
            u32::from_le_bytes(bytes[0..4].try_into().unwrap()),
            SCHEMA_VERSION
        );
        let boolean_index = bytes.len() - 1 - 2 * 4;
        assert_eq!(bytes[boolean_index], 1);
        assert_eq!(from_bytes::<EntityBreathe>(&bytes).unwrap(), original);
    }

    #[test]
    fn codec_rejects_unsupported_schema_invalid_boolean_and_trailing_bytes() {
        let bytes = to_bytes(&event()).unwrap();

        let mut bad_schema = bytes.clone();
        bad_schema[0..4].copy_from_slice(&2_u32.to_le_bytes());
        assert_eq!(
            from_bytes::<EntityBreathe>(&bad_schema).unwrap_err().code(),
            ErrorCode::IncompatibleAbiVersion
        );

        let mut bad_boolean = bytes.clone();
        let boolean_index = bad_boolean.len() - 1 - 2 * 4;
        bad_boolean[boolean_index] = 2;
        assert_eq!(
            from_bytes::<EntityBreathe>(&bad_boolean)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidBoolean
        );

        let mut trailing = bytes;
        trailing.push(0);
        assert!(from_bytes::<EntityBreathe>(&trailing).is_err());
    }

    #[test]
    fn published_limits_match_the_host_contract() {
        assert_eq!("entity.breathe", EVENT_NAME);
        assert_eq!("sjml_v1_event_entity.breathe", EXPORT_NAME);
        assert_eq!(1, SCHEMA_VERSION);
        assert_eq!(256, MAX_STRING_BYTES);
        assert_eq!(0, MIN_AIR_AMOUNT);
        assert_eq!(1_000_000, MAX_AIR_AMOUNT);
        assert_eq!(768, MAX_PAYLOAD_BYTES);
        assert_eq!(64, MAX_CALLBACKS);
    }
}
