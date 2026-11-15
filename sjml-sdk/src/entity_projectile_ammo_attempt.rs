//! Detached payload for cancellable logical-server projectile ammo selection.
//!
//! Register an `entity.projectile_ammo_attempt` callback with `events.subscribe`
//! and `events.cancel`. The callback is synchronous because NeoForge reads the
//! selected projectile stack immediately after the event returns.

use crate::{StableDecode, StableEncode};

/// Event name subscribed with `events::subscribe`.
pub const ENTITY_PROJECTILE_AMMO_ATTEMPT_EVENT_NAME: &str = "entity.projectile_ammo_attempt";
/// Cancellation grant required to veto ammo selection; `events.subscribe` is also required.
pub const ENTITY_PROJECTILE_AMMO_ATTEMPT_CANCEL_CAPABILITY: &str = "events.cancel";
/// Maximum UTF-8 byte length of an entity, type, dimension, or item identifier.
pub const MAX_ENTITY_PROJECTILE_AMMO_ATTEMPT_STRING_BYTES: usize = 256;
/// Maximum stack count accepted for one weapon or ammo snapshot.
pub const MAX_ENTITY_PROJECTILE_AMMO_ATTEMPT_ITEM_COUNT: u32 = 1_000_000;
/// Maximum number of subscribed `entity.projectile_ammo_attempt` handlers per selection.
pub const MAX_ENTITY_PROJECTILE_AMMO_ATTEMPT_CALLBACKS: usize = 64;
/// Maximum encoded payload size in bytes.
pub const MAX_ENTITY_PROJECTILE_AMMO_ATTEMPT_PAYLOAD_BYTES: usize = 2_048;

/// Synchronous cancellable snapshot of the weapon and selected ammo stacks.
///
/// NeoForge fires `entity.projectile_ammo_attempt` when a living entity asks
/// for the ammo stack used by a projectile weapon, for example before a bow
/// consumes an arrow. Return `Ok(true)` to clear the selected ammo, which the
/// host applies by setting the projectile stack to empty. The payload carries
/// only item identities and counts: callbacks cannot change either stack.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityProjectileAmmoAttempt {
    /// Canonical UUID of the living entity selecting ammo.
    pub entity_id: String,
    /// Registry ID of the entity type, such as `minecraft:zombie`.
    pub entity_type_id: String,
    /// Resource location of the dimension containing the entity.
    pub dimension_id: String,
    /// Projectile weapon stack that requested the ammo, such as `minecraft:bow`.
    pub weapon: AmmoItemSnapshot,
    /// Ammo stack vanilla selected, or `minecraft:air` with count 0 when none was found.
    pub ammo: AmmoItemSnapshot,
}

/// Detached registry item and bounded count from one weapon or ammo stack.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct AmmoItemSnapshot {
    /// Canonical registry ID of the item, or `minecraft:air` for an empty stack.
    pub item_id: String,
    /// Stack count, bounded by the host to `0..=1_000_000`; empty stacks use 0.
    pub count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_bytes, to_bytes, ErrorCode};

    fn event() -> EntityProjectileAmmoAttempt {
        EntityProjectileAmmoAttempt {
            entity_id: "00000000-0000-0000-0000-000000000001".into(),
            entity_type_id: "minecraft:zombie".into(),
            dimension_id: "minecraft:overworld".into(),
            weapon: AmmoItemSnapshot {
                item_id: "minecraft:bow".into(),
                count: 1,
            },
            ammo: AmmoItemSnapshot {
                item_id: "minecraft:arrow".into(),
                count: 1,
            },
        }
    }

    #[test]
    fn event_matches_the_java_schema_v1_wire_bytes() {
        let encoded = to_bytes(&event()).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_i32.to_le_bytes());
        for value in [
            "00000000-0000-0000-0000-000000000001",
            "minecraft:zombie",
            "minecraft:overworld",
        ] {
            expected.extend_from_slice(&(value.len() as u32).to_le_bytes());
            expected.extend_from_slice(value.as_bytes());
        }
        for (item_id, count) in [("minecraft:bow", 1_u32), ("minecraft:arrow", 1)] {
            expected.extend_from_slice(&1_i32.to_le_bytes());
            expected.extend_from_slice(&(item_id.len() as u32).to_le_bytes());
            expected.extend_from_slice(item_id.as_bytes());
            expected.extend_from_slice(&count.to_le_bytes());
        }
        assert_eq!(encoded, expected);
        assert_eq!(
            from_bytes::<EntityProjectileAmmoAttempt>(&encoded).unwrap(),
            event()
        );
    }

    #[test]
    fn empty_ammo_round_trips_as_air() {
        let mut event = event();
        event.ammo = AmmoItemSnapshot {
            item_id: "minecraft:air".into(),
            count: 0,
        };
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(
            from_bytes::<EntityProjectileAmmoAttempt>(&encoded).unwrap(),
            event
        );
    }

    #[test]
    fn unsupported_schema_versions_are_rejected() {
        let mut encoded = to_bytes(&event()).unwrap();
        encoded[..4].copy_from_slice(&2_i32.to_le_bytes());
        assert_eq!(
            from_bytes::<EntityProjectileAmmoAttempt>(&encoded)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );

        let event = event();
        let nested_schema = 4
            + (4 + event.entity_id.len())
            + (4 + event.entity_type_id.len())
            + (4 + event.dimension_id.len());
        let mut encoded = to_bytes(&event).unwrap();
        encoded[nested_schema..nested_schema + 4].copy_from_slice(&2_i32.to_le_bytes());
        assert_eq!(
            from_bytes::<EntityProjectileAmmoAttempt>(&encoded)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );
    }

    #[test]
    fn constants_match_the_host_contract() {
        assert_eq!(
            ENTITY_PROJECTILE_AMMO_ATTEMPT_EVENT_NAME,
            "entity.projectile_ammo_attempt"
        );
        assert_eq!(
            ENTITY_PROJECTILE_AMMO_ATTEMPT_CANCEL_CAPABILITY,
            "events.cancel"
        );
        assert_eq!(MAX_ENTITY_PROJECTILE_AMMO_ATTEMPT_STRING_BYTES, 256);
        assert_eq!(MAX_ENTITY_PROJECTILE_AMMO_ATTEMPT_ITEM_COUNT, 1_000_000);
        assert_eq!(MAX_ENTITY_PROJECTILE_AMMO_ATTEMPT_CALLBACKS, 64);
        assert_eq!(MAX_ENTITY_PROJECTILE_AMMO_ATTEMPT_PAYLOAD_BYTES, 2_048);
    }
}
