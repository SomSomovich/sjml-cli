//! Detached payload and veto response for logical-server mob griefing checks.
//!
//! Register an `entity.mob_griefing_attempt` callback with `events.subscribe` and
//! `events.cancel`. Returning `true` denies the griefing action, so vanilla skips behavior
//! such as an enderman picking up a block. The callback is synchronous because vanilla reads
//! the resulting `canGrief` flag immediately, so `#[event_async]` handlers are rejected.

use crate::{StableDecode, StableEncode};

/// Event name for synchronous logical-server mob griefing attempts.
pub const ENTITY_MOB_GRIEFING_ATTEMPT_EVENT_NAME: &str = "entity.mob_griefing_attempt";
/// Guest export name the host resolves for one mob griefing callback.
pub const ENTITY_MOB_GRIEFING_ATTEMPT_EXPORT_NAME: &str =
    "sjml_v1_event_entity.mob_griefing_attempt";
/// Cancellation grant required to deny mob griefing; `events.subscribe` is also required.
pub const ENTITY_MOB_GRIEFING_ATTEMPT_CANCEL_CAPABILITY: &str = "events.cancel";
/// Schema version of the detached mob griefing payload.
pub const ENTITY_MOB_GRIEFING_ATTEMPT_SCHEMA_VERSION: u32 = 1;
/// Maximum UTF-8 byte length of an entity UUID, type ID, or dimension ID.
pub const MAX_ENTITY_MOB_GRIEFING_ATTEMPT_STRING_BYTES: usize = 256;
/// Maximum callbacks the host invokes for one mob griefing attempt.
pub const MAX_ENTITY_MOB_GRIEFING_ATTEMPT_CALLBACKS: usize = 64;
/// Maximum encoded payload size for one mob griefing attempt callback.
pub const MAX_ENTITY_MOB_GRIEFING_ATTEMPT_PAYLOAD_BYTES: usize = 1_024;

/// Synchronous snapshot captured when vanilla checks the mob griefing rule.
///
/// NeoForge fires `entity.mob_griefing_attempt` through `EntityMobGriefingEvent`, a
/// non-cancellable event consulted whenever the `mobGriefing` game rule is used. Return
/// `Ok(true)` to deny the griefing action: the host calls `setCanGrief(false)` before vanilla
/// acts. Return `Ok(false)` to keep the rule value. `griefing_enabled` carries the game rule
/// value observed at capture time, so a disabled rule already reports `false` and guests
/// cannot re-enable griefing. All values are detached; no `Entity`, level, or host handle
/// crosses the guest ABI. Both `events.subscribe` and `events.cancel` are required, and the
/// callback must remain synchronous.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityMobGriefingAttempt {
    /// Canonical lowercase UUID of the entity that would perform the griefing action.
    pub entity_id: String,
    /// Registry ID of the entity type, such as `minecraft:zombie`.
    pub entity_type_id: String,
    /// Resource ID of the dimension containing the entity.
    pub dimension_id: String,
    /// Value of the `mobGriefing` game rule at capture time.
    pub griefing_enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_bytes, to_bytes, ErrorCode};

    fn event() -> EntityMobGriefingAttempt {
        EntityMobGriefingAttempt {
            entity_id: "68ccfd1a-588d-4eb4-a0c9-31046942d74f".into(),
            entity_type_id: "minecraft:zombie".into(),
            dimension_id: "minecraft:overworld".into(),
            griefing_enabled: true,
        }
    }

    fn push_string(bytes: &mut Vec<u8>, value: &str) {
        bytes.extend_from_slice(&(value.len() as u32).to_le_bytes());
        bytes.extend_from_slice(value.as_bytes());
    }

    #[test]
    fn encodes_the_java_schema_v1_wire_bytes() {
        let mut expected = 1_u32.to_le_bytes().to_vec();
        push_string(&mut expected, "68ccfd1a-588d-4eb4-a0c9-31046942d74f");
        push_string(&mut expected, "minecraft:zombie");
        push_string(&mut expected, "minecraft:overworld");
        expected.push(1);
        assert_eq!(to_bytes(&event()).unwrap(), expected);
    }

    #[test]
    fn event_round_trips_the_detached_payload() {
        let event = event();
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(
            &encoded[..4],
            &ENTITY_MOB_GRIEFING_ATTEMPT_SCHEMA_VERSION.to_le_bytes()
        );
        assert_eq!(
            from_bytes::<EntityMobGriefingAttempt>(&encoded).unwrap(),
            event
        );
    }

    #[test]
    fn event_constants_match_the_host_contract() {
        assert_eq!(
            ENTITY_MOB_GRIEFING_ATTEMPT_EVENT_NAME,
            "entity.mob_griefing_attempt"
        );
        assert_eq!(
            ENTITY_MOB_GRIEFING_ATTEMPT_EXPORT_NAME,
            "sjml_v1_event_entity.mob_griefing_attempt"
        );
        assert_eq!(
            ENTITY_MOB_GRIEFING_ATTEMPT_CANCEL_CAPABILITY,
            "events.cancel"
        );
        assert_eq!(MAX_ENTITY_MOB_GRIEFING_ATTEMPT_STRING_BYTES, 256);
        assert_eq!(MAX_ENTITY_MOB_GRIEFING_ATTEMPT_CALLBACKS, 64);
        assert_eq!(MAX_ENTITY_MOB_GRIEFING_ATTEMPT_PAYLOAD_BYTES, 1_024);
    }

    #[test]
    fn malformed_schema_and_boolean_values_are_rejected() {
        let mut wrong_schema = to_bytes(&event()).unwrap();
        wrong_schema[..4].copy_from_slice(&2_u32.to_le_bytes());
        assert_eq!(
            from_bytes::<EntityMobGriefingAttempt>(&wrong_schema)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );

        let mut wrong_boolean = to_bytes(&event()).unwrap();
        *wrong_boolean.last_mut().unwrap() = 2;
        assert_eq!(
            from_bytes::<EntityMobGriefingAttempt>(&wrong_boolean)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidBoolean
        );
    }
}
