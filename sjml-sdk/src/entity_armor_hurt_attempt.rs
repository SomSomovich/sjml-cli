//! Detached payload and response for cancellable logical-server armor-hurt attempts.
//!
//! Register an `entity.armor_hurt_attempt` callback with `events.subscribe` and
//! `events.cancel`. Returning `Ok(true)` cancels NeoForge's `ArmorHurtEvent`,
//! so none of the entity's armor pieces lose durability from the hit. The
//! callback is synchronous because vanilla applies the durability loss
//! immediately after the event returns.

use crate::{StableDecode, StableEncode};

/// Event name for armor durability damage on the logical server.
pub const ENTITY_ARMOR_HURT_ATTEMPT_EVENT_NAME: &str = "entity.armor_hurt_attempt";
/// Guest export the host invokes for one `entity.armor_hurt_attempt` callback.
pub const ENTITY_ARMOR_HURT_ATTEMPT_EXPORT_NAME: &str = "sjml_v1_event_entity.armor_hurt_attempt";
/// Cancellation grant required to veto armor damage; `events.subscribe` is also required.
pub const ENTITY_ARMOR_HURT_ATTEMPT_CANCEL_CAPABILITY: &str = "events.cancel";
/// Schema version of the detached armor-hurt payload.
pub const ENTITY_ARMOR_HURT_ATTEMPT_SCHEMA_VERSION: u32 = 1;
/// Maximum UTF-8 byte length of an entity UUID, entity type, or dimension.
pub const MAX_ENTITY_ARMOR_HURT_ATTEMPT_STRING_BYTES: usize = 256;
/// Maximum UTF-8 byte length of the damaging source message ID.
pub const MAX_ENTITY_ARMOR_HURT_ATTEMPT_DAMAGE_SOURCE_BYTES: usize = 64;
/// Maximum encoded payload size for one armor-hurt callback.
pub const MAX_ENTITY_ARMOR_HURT_ATTEMPT_PAYLOAD_BYTES: usize = 1_024;
/// Maximum callbacks invoked for one armor-hurt attempt.
pub const MAX_ENTITY_ARMOR_HURT_ATTEMPT_CALLBACKS: usize = 64;

/// Synchronous cancellable snapshot captured when armor would lose durability.
///
/// Returning `Ok(true)` from the handler cancels NeoForge's `ArmorHurtEvent`,
/// so the entity's armor takes no durability loss from this hit. Returning
/// `Ok(false)` keeps vanilla durability behavior. The detached snapshot
/// contains the entity UUID and type, the dimension, and the damage source
/// message ID. All values are detached; no `LivingEntity`, level, or host
/// handle crosses the guest ABI.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityArmorHurtAttempt {
    /// Canonical UUID of the entity whose armor would be damaged.
    pub entity_id: String,
    /// Registry ID of the entity type, such as `minecraft:zombie`.
    pub entity_type_id: String,
    /// Resource location of the dimension containing the entity.
    pub dimension_id: String,
    /// Damage source message ID, such as `death.attack.generic`.
    pub damage_source: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_bytes, to_bytes, ErrorCode};

    fn sample() -> EntityArmorHurtAttempt {
        EntityArmorHurtAttempt {
            entity_id: "00000000-0000-0000-0000-000000000001".into(),
            entity_type_id: "minecraft:zombie".into(),
            dimension_id: "minecraft:overworld".into(),
            damage_source: "death.attack.generic".into(),
        }
    }

    fn append_string(output: &mut Vec<u8>, value: &str) {
        output.extend_from_slice(&(value.len() as u32).to_le_bytes());
        output.extend_from_slice(value.as_bytes());
    }

    #[test]
    fn armor_hurt_attempt_uses_the_stable_schema_v1_wire_layout() {
        let event = sample();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_string(&mut expected, &event.entity_id);
        append_string(&mut expected, &event.entity_type_id);
        append_string(&mut expected, &event.dimension_id);
        append_string(&mut expected, &event.damage_source);

        assert_eq!(to_bytes(&event).unwrap(), expected);
        assert_eq!(
            from_bytes::<EntityArmorHurtAttempt>(&expected).unwrap(),
            event
        );
    }

    #[test]
    fn armor_hurt_attempt_round_trips_other_damage_sources() {
        let mut event = sample();
        event.entity_type_id = "minecraft:player".into();
        event.dimension_id = "minecraft:the_nether".into();
        event.damage_source = "death.attack.player".into();
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(
            from_bytes::<EntityArmorHurtAttempt>(&encoded).unwrap(),
            event
        );
    }

    #[test]
    fn armor_hurt_attempt_rejects_other_schema_versions() {
        let mut encoded = to_bytes(&sample()).unwrap();
        encoded[..4].copy_from_slice(&2_i32.to_le_bytes());
        assert_eq!(
            from_bytes::<EntityArmorHurtAttempt>(&encoded)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );
    }

    #[test]
    fn armor_hurt_attempt_constants_match_the_host_contract() {
        assert_eq!(
            ENTITY_ARMOR_HURT_ATTEMPT_EVENT_NAME,
            "entity.armor_hurt_attempt"
        );
        assert_eq!(
            ENTITY_ARMOR_HURT_ATTEMPT_EXPORT_NAME,
            "sjml_v1_event_entity.armor_hurt_attempt"
        );
        assert_eq!(ENTITY_ARMOR_HURT_ATTEMPT_CANCEL_CAPABILITY, "events.cancel");
        assert_eq!(ENTITY_ARMOR_HURT_ATTEMPT_SCHEMA_VERSION, 1);
        assert_eq!(MAX_ENTITY_ARMOR_HURT_ATTEMPT_STRING_BYTES, 256);
        assert_eq!(MAX_ENTITY_ARMOR_HURT_ATTEMPT_DAMAGE_SOURCE_BYTES, 64);
        assert_eq!(MAX_ENTITY_ARMOR_HURT_ATTEMPT_PAYLOAD_BYTES, 1_024);
        assert_eq!(MAX_ENTITY_ARMOR_HURT_ATTEMPT_CALLBACKS, 64);
    }
}
