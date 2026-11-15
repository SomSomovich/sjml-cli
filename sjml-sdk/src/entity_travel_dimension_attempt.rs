//! Detached payload and veto response for cancellable logical-server dimension transfers.
//!
//! Register an `entity.travel_dimension_attempt` callback with `events.subscribe` and
//! `events.cancel`. Returning `true` keeps the entity in its current level. The callback is
//! synchronous because vanilla applies the transfer immediately after the event returns, so
//! `#[event_async]` handlers are rejected.

use crate::{StableDecode, StableEncode};

/// Event name for cancellable logical-server dimension travel attempts.
pub const ENTITY_TRAVEL_DIMENSION_ATTEMPT_EVENT_NAME: &str = "entity.travel_dimension_attempt";
/// Cancellation grant required to veto a dimension transfer; `events.subscribe` is also required.
pub const ENTITY_TRAVEL_DIMENSION_ATTEMPT_CANCEL_CAPABILITY: &str = "events.cancel";
/// Schema version of the detached dimension travel payload.
pub const ENTITY_TRAVEL_DIMENSION_ATTEMPT_SCHEMA_VERSION: u32 = 1;
/// Maximum UTF-8 byte length of an entity UUID, type ID, or dimension ID.
pub const MAX_ENTITY_TRAVEL_DIMENSION_ATTEMPT_STRING_BYTES: usize = 256;
/// Maximum callbacks the host invokes for one dimension travel attempt.
pub const MAX_ENTITY_TRAVEL_DIMENSION_ATTEMPT_CALLBACKS: usize = 64;
/// Maximum encoded payload size for one dimension travel attempt callback.
pub const MAX_ENTITY_TRAVEL_DIMENSION_ATTEMPT_PAYLOAD_BYTES: usize = 1_024;

/// Synchronous cancellable snapshot captured before a server entity changes dimension.
///
/// NeoForge fires `entity.travel_dimension_attempt` before an entity travels to another
/// dimension. Return `Ok(true)` to cancel the transfer, so the entity stays in its current
/// level, or `Ok(false)` to keep vanilla behavior. All values are detached; no `Entity`,
/// level, or host handle crosses the guest ABI. Both `events.subscribe` and `events.cancel`
/// are required, and the callback must remain synchronous.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityTravelDimensionAttempt {
    /// Canonical lowercase UUID of the entity that is about to travel.
    pub entity_id: String,
    /// Registry ID of the traveling entity type.
    pub entity_type_id: String,
    /// Resource ID of the dimension the entity currently occupies.
    pub from_dimension_id: String,
    /// Resource ID of the dimension the entity would enter.
    pub to_dimension_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_bytes, to_bytes, ErrorCode};

    fn event() -> EntityTravelDimensionAttempt {
        EntityTravelDimensionAttempt {
            entity_id: "68ccfd1a-588d-4eb4-a0c9-31046942d74f".into(),
            entity_type_id: "minecraft:player".into(),
            from_dimension_id: "minecraft:overworld".into(),
            to_dimension_id: "minecraft:the_nether".into(),
        }
    }

    fn expected_wire(event: &EntityTravelDimensionAttempt) -> Vec<u8> {
        let mut output = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut output, &event.entity_id);
        append_test_string(&mut output, &event.entity_type_id);
        append_test_string(&mut output, &event.from_dimension_id);
        append_test_string(&mut output, &event.to_dimension_id);
        output
    }

    #[test]
    fn dimension_travel_attempt_matches_the_java_wire_layout_and_round_trips() {
        let event = event();
        let expected = expected_wire(&event);
        assert_eq!(to_bytes(&event).unwrap(), expected);
        assert_eq!(
            from_bytes::<EntityTravelDimensionAttempt>(&expected).unwrap(),
            event
        );
    }

    #[test]
    fn dimension_travel_attempt_rejects_a_foreign_schema_version() {
        let mut wrong_version = to_bytes(&event()).unwrap();
        wrong_version[..4].copy_from_slice(&2_i32.to_le_bytes());
        assert_eq!(
            from_bytes::<EntityTravelDimensionAttempt>(&wrong_version)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );
    }

    #[test]
    fn event_constants_match_the_host_contract() {
        assert_eq!(
            ENTITY_TRAVEL_DIMENSION_ATTEMPT_EVENT_NAME,
            "entity.travel_dimension_attempt"
        );
        assert_eq!(
            ENTITY_TRAVEL_DIMENSION_ATTEMPT_CANCEL_CAPABILITY,
            "events.cancel"
        );
        assert_eq!(ENTITY_TRAVEL_DIMENSION_ATTEMPT_SCHEMA_VERSION, 1);
        assert_eq!(MAX_ENTITY_TRAVEL_DIMENSION_ATTEMPT_STRING_BYTES, 256);
        assert_eq!(MAX_ENTITY_TRAVEL_DIMENSION_ATTEMPT_CALLBACKS, 64);
        assert_eq!(MAX_ENTITY_TRAVEL_DIMENSION_ATTEMPT_PAYLOAD_BYTES, 1_024);
    }

    fn append_test_string(output: &mut Vec<u8>, value: &str) {
        output.extend_from_slice(&(value.len() as i32).to_le_bytes());
        output.extend_from_slice(value.as_bytes());
    }
}
