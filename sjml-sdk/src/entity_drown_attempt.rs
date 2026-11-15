//! Detached payload and response for cancellable logical-server drowning attempts.
//!
//! Register an `entity.drown_attempt` callback with `events.subscribe` and
//! `events.cancel`. Returning `Ok(true)` cancels NeoForge's `LivingDrownEvent`,
//! so the entity takes no drowning damage and loses no bubbles this tick. The
//! callback is synchronous because vanilla reads the event immediately.

use crate::{StableDecode, StableEncode};

/// Event name for drowning attempts on the logical server.
pub const ENTITY_DROWN_ATTEMPT_EVENT_NAME: &str = "entity.drown_attempt";
/// Guest export the host invokes for one `entity.drown_attempt` callback.
pub const ENTITY_DROWN_ATTEMPT_EXPORT_NAME: &str = "sjml_v1_event_entity.drown_attempt";
/// Cancellation grant required to veto drowning; `events.subscribe` is also required.
pub const ENTITY_DROWN_ATTEMPT_CANCEL_CAPABILITY: &str = "events.cancel";
/// Schema version of the detached drowning payload.
pub const ENTITY_DROWN_ATTEMPT_SCHEMA_VERSION: u32 = 1;
/// Maximum UTF-8 byte length of an entity UUID, entity type, or dimension.
pub const MAX_ENTITY_DROWN_ATTEMPT_STRING_BYTES: usize = 256;
/// Maximum drowning damage accepted by the payload schema.
pub const MAX_ENTITY_DROWN_ATTEMPT_DAMAGE: f32 = 1_000.0;
/// Maximum bubble count accepted by the payload schema.
pub const MAX_ENTITY_DROWN_ATTEMPT_BUBBLES: i32 = 1_000;
/// Maximum encoded payload size for one drowning callback.
pub const MAX_ENTITY_DROWN_ATTEMPT_PAYLOAD_BYTES: usize = 1_024;
/// Maximum callbacks invoked for one drowning attempt.
pub const MAX_ENTITY_DROWN_ATTEMPT_CALLBACKS: usize = 64;

/// Synchronous cancellable snapshot captured when a living entity drowns.
///
/// Returning `Ok(true)` from the handler cancels NeoForge's
/// `LivingDrownEvent`, so the entity neither takes drowning damage nor loses
/// bubbles this tick. Returning `Ok(false)` keeps vanilla drowning behavior.
/// The detached snapshot contains the entity UUID and type, the dimension, the
/// active-drowning flag, the pending damage amount, and the pending bubble
/// count. All values are detached; no `LivingEntity`, level, or host handle
/// crosses the guest ABI.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityDrownAttempt {
    /// Canonical UUID of the entity that would drown.
    pub entity_id: String,
    /// Registry ID of the entity type, such as `minecraft:zombie`.
    pub entity_type_id: String,
    /// Resource location of the dimension containing the entity.
    pub dimension_id: String,
    /// True when the entity is actively drowning and would take damage.
    pub is_drowning: bool,
    /// Pending drowning damage, finite and within `0.0..=1_000.0`.
    pub damage_amount: f32,
    /// Pending bubble count, within `0..=1_000`.
    pub bubble_count: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_bytes, to_bytes, ErrorCode};

    fn sample() -> EntityDrownAttempt {
        EntityDrownAttempt {
            entity_id: "00000000-0000-0000-0000-000000000001".into(),
            entity_type_id: "minecraft:zombie".into(),
            dimension_id: "minecraft:overworld".into(),
            is_drowning: true,
            damage_amount: 2.0,
            bubble_count: 5,
        }
    }

    fn append_string(output: &mut Vec<u8>, value: &str) {
        output.extend_from_slice(&(value.len() as u32).to_le_bytes());
        output.extend_from_slice(value.as_bytes());
    }

    #[test]
    fn drown_attempt_uses_the_stable_schema_v1_wire_layout() {
        let event = sample();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_string(&mut expected, &event.entity_id);
        append_string(&mut expected, &event.entity_type_id);
        append_string(&mut expected, &event.dimension_id);
        expected.push(1);
        expected.extend_from_slice(&event.damage_amount.to_bits().to_le_bytes());
        expected.extend_from_slice(&event.bubble_count.to_le_bytes());

        assert_eq!(to_bytes(&event).unwrap(), expected);
        assert_eq!(from_bytes::<EntityDrownAttempt>(&expected).unwrap(), event);
    }

    #[test]
    fn drown_attempt_round_trips_a_quiet_tick() {
        let mut event = sample();
        event.entity_type_id = "minecraft:cow".into();
        event.is_drowning = false;
        event.damage_amount = 0.0;
        event.bubble_count = 0;
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(from_bytes::<EntityDrownAttempt>(&encoded).unwrap(), event);
    }

    #[test]
    fn drown_attempt_rejects_other_schema_versions() {
        let mut encoded = to_bytes(&sample()).unwrap();
        encoded[..4].copy_from_slice(&2_i32.to_le_bytes());
        assert_eq!(
            from_bytes::<EntityDrownAttempt>(&encoded)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );
    }

    #[test]
    fn drown_attempt_rejects_invalid_boolean_bytes() {
        let event = sample();
        let mut encoded = to_bytes(&event).unwrap();
        let flag = 4
            + (4 + event.entity_id.len())
            + (4 + event.entity_type_id.len())
            + (4 + event.dimension_id.len());
        encoded[flag] = 2;
        assert_eq!(
            from_bytes::<EntityDrownAttempt>(&encoded)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidBoolean
        );
    }

    #[test]
    fn drown_attempt_constants_match_the_host_contract() {
        assert_eq!(ENTITY_DROWN_ATTEMPT_EVENT_NAME, "entity.drown_attempt");
        assert_eq!(
            ENTITY_DROWN_ATTEMPT_EXPORT_NAME,
            "sjml_v1_event_entity.drown_attempt"
        );
        assert_eq!(ENTITY_DROWN_ATTEMPT_CANCEL_CAPABILITY, "events.cancel");
        assert_eq!(ENTITY_DROWN_ATTEMPT_SCHEMA_VERSION, 1);
        assert_eq!(MAX_ENTITY_DROWN_ATTEMPT_STRING_BYTES, 256);
        assert_eq!(MAX_ENTITY_DROWN_ATTEMPT_DAMAGE, 1_000.0);
        assert_eq!(MAX_ENTITY_DROWN_ATTEMPT_BUBBLES, 1_000);
        assert_eq!(MAX_ENTITY_DROWN_ATTEMPT_PAYLOAD_BYTES, 1_024);
        assert_eq!(MAX_ENTITY_DROWN_ATTEMPT_CALLBACKS, 64);
    }
}
