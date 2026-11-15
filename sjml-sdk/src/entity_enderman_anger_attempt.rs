//! Detached payload and veto for cancellable logical-server enderman anger.
//!
//! Register an `entity.enderman_anger_attempt` callback with `events.subscribe`
//! and `events.cancel`. Returning `Ok(true)` cancels NeoForge's
//! `EnderManAngerEvent`, so the enderman does not target the player. The
//! callback is synchronous because the result is read as soon as the event
//! returns.

use crate::{StableDecode, StableEncode};

/// Event name subscribed with `events::subscribe`.
pub const ENTITY_ENDERMAN_ANGER_ATTEMPT_EVENT_NAME: &str = "entity.enderman_anger_attempt";
/// Cancellation grant required to veto enderman anger; `events.subscribe` is also required.
pub const ENTITY_ENDERMAN_ANGER_ATTEMPT_CANCEL_CAPABILITY: &str = "events.cancel";
/// Maximum UTF-8 byte length of one UUID, entity type, or dimension identifier.
pub const MAX_ENTITY_ENDERMAN_ANGER_ATTEMPT_ID_BYTES: usize = 256;
/// Maximum serialized payload size for one enderman-anger callback.
pub const MAX_ENTITY_ENDERMAN_ANGER_ATTEMPT_PAYLOAD_BYTES: usize = 1_024;
/// Maximum callbacks invoked for one enderman anger attempt.
pub const MAX_ENTITY_ENDERMAN_ANGER_ATTEMPT_CALLBACKS: usize = 64;

/// Synchronous cancellable snapshot captured before an enderman targets a player.
///
/// Returning `Ok(true)` from the handler cancels the event: the enderman does
/// not target the player that looked at it. Returning `Ok(false)` keeps vanilla
/// targeting. The detached snapshot contains the enderman UUID and type, the
/// target player UUID, and the dimension; no Minecraft objects cross the ABI.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityEndermanAngerAttempt {
    /// Canonical UUID of the angered enderman.
    pub entity_id: String,
    /// Registry ID of the enderman type, normally `minecraft:enderman`.
    pub entity_type_id: String,
    /// Canonical UUID of the player the enderman would target.
    pub player_id: String,
    /// Resource location of the dimension containing the enderman.
    pub dimension_id: String,
}

#[cfg(test)]
mod tests {
    use super::{
        EntityEndermanAngerAttempt, MAX_ENTITY_ENDERMAN_ANGER_ATTEMPT_CALLBACKS,
        MAX_ENTITY_ENDERMAN_ANGER_ATTEMPT_ID_BYTES,
        MAX_ENTITY_ENDERMAN_ANGER_ATTEMPT_PAYLOAD_BYTES,
    };
    use crate::{from_bytes, to_bytes, ErrorCode};

    fn sample() -> EntityEndermanAngerAttempt {
        EntityEndermanAngerAttempt {
            entity_id: "00000000-0000-0000-0000-000000000001".into(),
            entity_type_id: "minecraft:enderman".into(),
            player_id: "00000000-0000-0000-0000-000000000002".into(),
            dimension_id: "minecraft:overworld".into(),
        }
    }

    #[test]
    fn enderman_anger_attempt_matches_the_java_schema_v1_wire_layout() {
        let event = sample();
        let encoded = to_bytes(&event).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_u32.to_le_bytes());
        for value in [
            &event.entity_id,
            &event.entity_type_id,
            &event.player_id,
            &event.dimension_id,
        ] {
            expected.extend_from_slice(&(value.len() as u32).to_le_bytes());
            expected.extend_from_slice(value.as_bytes());
        }
        assert_eq!(encoded, expected);
        assert_eq!(
            from_bytes::<EntityEndermanAngerAttempt>(&encoded).unwrap(),
            event
        );
    }

    #[test]
    fn enderman_anger_attempt_round_trips_other_dimensions() {
        let mut event = sample();
        event.dimension_id = "minecraft:the_end".into();
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(
            from_bytes::<EntityEndermanAngerAttempt>(&encoded).unwrap(),
            event
        );
    }

    #[test]
    fn enderman_anger_attempt_rejects_other_schema_versions() {
        let mut encoded = to_bytes(&sample()).unwrap();
        encoded[..4].copy_from_slice(&2_u32.to_le_bytes());
        assert_eq!(
            from_bytes::<EntityEndermanAngerAttempt>(&encoded)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );
    }

    #[test]
    fn enderman_anger_attempt_constants_match_the_host_contract() {
        assert_eq!(
            super::ENTITY_ENDERMAN_ANGER_ATTEMPT_EVENT_NAME,
            "entity.enderman_anger_attempt"
        );
        assert_eq!(
            super::ENTITY_ENDERMAN_ANGER_ATTEMPT_CANCEL_CAPABILITY,
            "events.cancel"
        );
        assert_eq!(MAX_ENTITY_ENDERMAN_ANGER_ATTEMPT_ID_BYTES, 256);
        assert_eq!(MAX_ENTITY_ENDERMAN_ANGER_ATTEMPT_PAYLOAD_BYTES, 1_024);
        assert_eq!(MAX_ENTITY_ENDERMAN_ANGER_ATTEMPT_CALLBACKS, 64);
    }
}
