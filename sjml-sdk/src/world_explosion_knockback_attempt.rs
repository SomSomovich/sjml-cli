//! Detached observation of the logical-server explosion knockback decision.
//!
//! Subscribe with `events::subscribe(EVENT_NAME, EventPriority::Normal)` and a typed
//! `#[event(name = "world.explosion_knockback_attempt")]` callback returning `SjmlResult<bool>`.
//! Return `Ok(true)` to veto the knockback: the host sets the event's knockback velocity to zero,
//! so the affected entity receives no explosion impulse. Both `events.subscribe` and
//! `events.cancel` are required, and the callback must remain synchronous. The host fires the
//! event on the owning logical-server thread and invokes at most [`MAX_CALLBACKS`] callbacks per
//! attempt.

use sjml::{StableDecode, StableEncode};

/// Event name used with [`crate::events::subscribe`].
pub const EVENT_NAME: &str = "world.explosion_knockback_attempt";
/// Generated ABI export name for the typed explosion-knockback callback.
pub const EXPORT_NAME: &str = "sjml_v1_event_world.explosion_knockback_attempt";
/// Maximum UTF-8 byte length of each identifier string.
pub const MAX_STRING_BYTES: usize = 256;
/// Maximum encoded bytes for one detached explosion-knockback event.
pub const MAX_PAYLOAD_BYTES: usize = 1_024;
/// Maximum guest callbacks invoked for one explosion-knockback attempt.
pub const MAX_CALLBACKS: usize = 64;
/// Maximum number of blocks affected by the explosion.
pub const MAX_AFFECTED_BLOCKS: u32 = 4_096;
/// Maximum absolute knockback velocity per axis.
pub const MAX_VELOCITY: f64 = 1_000.0;

/// Detached explosion knockback for one entity. It contains no level, entity, or host handle.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct WorldExplosionKnockbackAttempt {
    /// Resource location of the dimension containing the explosion.
    pub dimension_id: String,
    /// Canonical UUID of the entity receiving the knockback.
    pub entity_id: String,
    /// Registry ID of the affected entity type.
    pub entity_type_id: String,
    /// Number of blocks the explosion would affect, in `0..=4096`.
    pub affected_block_count: u32,
    /// Knockback velocity added on the x axis.
    pub vx: f64,
    /// Knockback velocity added on the y axis.
    pub vy: f64,
    /// Knockback velocity added on the z axis.
    pub vz: f64,
}

#[cfg(test)]
mod tests {
    use super::{
        WorldExplosionKnockbackAttempt, EVENT_NAME, EXPORT_NAME, MAX_AFFECTED_BLOCKS,
        MAX_CALLBACKS, MAX_PAYLOAD_BYTES, MAX_STRING_BYTES, MAX_VELOCITY,
    };
    use sjml::{from_bytes, to_bytes, ErrorCode};

    fn event() -> WorldExplosionKnockbackAttempt {
        WorldExplosionKnockbackAttempt {
            dimension_id: "minecraft:overworld".into(),
            entity_id: "bbcdf463-911f-4657-b469-8b0bed572ba5".into(),
            entity_type_id: "minecraft:zombie".into(),
            affected_block_count: 3,
            vx: 1.25,
            vy: -0.5,
            vz: 2.0,
        }
    }

    fn append_string(output: &mut Vec<u8>, value: &str) {
        output.extend_from_slice(&(value.len() as u32).to_le_bytes());
        output.extend_from_slice(value.as_bytes());
    }

    #[test]
    fn wire_bytes_match_schema_v1_field_order() {
        let original = event();
        let bytes = to_bytes(&original).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_u32.to_le_bytes());
        append_string(&mut expected, &original.dimension_id);
        append_string(&mut expected, &original.entity_id);
        append_string(&mut expected, &original.entity_type_id);
        expected.extend_from_slice(&original.affected_block_count.to_le_bytes());
        expected.extend_from_slice(&original.vx.to_le_bytes());
        expected.extend_from_slice(&original.vy.to_le_bytes());
        expected.extend_from_slice(&original.vz.to_le_bytes());
        assert_eq!(bytes, expected);
        assert!(bytes.len() <= MAX_PAYLOAD_BYTES);
    }

    #[test]
    fn stable_round_trip_preserves_signs_and_boundaries() {
        for event in [
            event(),
            WorldExplosionKnockbackAttempt {
                dimension_id: "minecraft:the_end".into(),
                entity_type_id: "minecraft:creeper".into(),
                affected_block_count: MAX_AFFECTED_BLOCKS,
                vx: MAX_VELOCITY,
                vy: -MAX_VELOCITY,
                vz: 0.0,
                ..event()
            },
        ] {
            let bytes = to_bytes(&event).unwrap();
            assert_eq!(
                from_bytes::<WorldExplosionKnockbackAttempt>(&bytes).unwrap(),
                event
            );
        }
    }

    #[test]
    fn codec_rejects_unsupported_schema_and_trailing_bytes() {
        let bytes = to_bytes(&event()).unwrap();

        let mut bad_schema = bytes.clone();
        bad_schema[0..4].copy_from_slice(&2_u32.to_le_bytes());
        assert_eq!(
            from_bytes::<WorldExplosionKnockbackAttempt>(&bad_schema)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );

        let mut trailing = bytes;
        trailing.push(0);
        assert!(from_bytes::<WorldExplosionKnockbackAttempt>(&trailing).is_err());
    }

    #[test]
    fn published_limits_match_the_host_contract() {
        assert_eq!("world.explosion_knockback_attempt", EVENT_NAME);
        assert_eq!(
            "sjml_v1_event_world.explosion_knockback_attempt",
            EXPORT_NAME
        );
        assert_eq!(256, MAX_STRING_BYTES);
        assert_eq!(1_024, MAX_PAYLOAD_BYTES);
        assert_eq!(64, MAX_CALLBACKS);
        assert_eq!(4_096, MAX_AFFECTED_BLOCKS);
        assert_eq!(1_000.0, MAX_VELOCITY);
    }
}
