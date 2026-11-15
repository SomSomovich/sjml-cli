//! Detached payload and response for cancellable logical-server item expiry attempts.
//!
//! Register an `item.expire_attempt` callback with `events.subscribe` and
//! `events.cancel`. Returning `Ok(true)` makes the host set the event's extra
//! life to `Short::MAX_VALUE`, which keeps the dropped item entity alive for
//! another 32,767 ticks. Returning `Ok(false)` leaves the extra life at zero so
//! vanilla expiry applies. The callback is synchronous because the extra life is
//! read as soon as the event returns.

use crate::{StableDecode, StableEncode};

/// Event name for item expiry attempts on the logical server.
pub const ITEM_EXPIRE_ATTEMPT_EVENT_NAME: &str = "item.expire_attempt";
/// Cancellation grant required to extend item lifetimes; `events.subscribe` is also required.
pub const ITEM_EXPIRE_ATTEMPT_CANCEL_CAPABILITY: &str = "events.cancel";
/// Maximum UTF-8 byte length for an item-entity, item, or dimension identifier.
pub const MAX_ITEM_EXPIRE_ATTEMPT_ID_BYTES: usize = 256;
/// Maximum dropped stack count accepted in one expiry attempt.
pub const MAX_ITEM_EXPIRE_ATTEMPT_ITEM_COUNT: i32 = 1_000_000;
/// Maximum current entity age in ticks accepted in one expiry attempt.
pub const MAX_ITEM_EXPIRE_ATTEMPT_AGE: i32 = 1_000_000;
/// Maximum world coordinate accepted in one item position.
pub const MAX_ITEM_EXPIRE_ATTEMPT_COORDINATE: f64 = 30_000_000.0;
/// Maximum serialized payload size for one item-expiry callback.
pub const MAX_ITEM_EXPIRE_ATTEMPT_PAYLOAD_BYTES: usize = 1_024;
/// Maximum callbacks invoked for one item expiry attempt.
pub const MAX_ITEM_EXPIRE_ATTEMPT_CALLBACKS: usize = 64;

/// Synchronous snapshot captured when a dropped item entity reaches its lifespan.
///
/// Returning `Ok(true)` from the handler extends the item's lifetime instead of
/// letting it despawn. Returning `Ok(false)` keeps vanilla expiry behavior. The
/// detached snapshot contains the item-entity UUID, the dropped item's registry
/// ID and stack count, the dimension, the bounded world position, and the
/// current entity age in ticks.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct ItemExpireAttempt {
    /// Canonical UUID of the expiring item entity.
    pub item_entity_id: String,
    /// Registry ID of the dropped item, such as `minecraft:diamond`.
    pub item_id: String,
    /// Size of the dropped item stack.
    pub count: i32,
    /// Resource location of the dimension containing the item entity.
    pub dimension_id: String,
    /// World-space X coordinate of the item entity.
    pub x: f64,
    /// World-space Y coordinate of the item entity.
    pub y: f64,
    /// World-space Z coordinate of the item entity.
    pub z: f64,
    /// Current age of the item entity in ticks.
    pub age: i32,
}

#[cfg(test)]
mod tests {
    use super::ItemExpireAttempt;
    use crate::{from_bytes, to_bytes, ErrorCode};

    fn sample() -> ItemExpireAttempt {
        ItemExpireAttempt {
            item_entity_id: "00000000-0000-0000-0000-000000000002".into(),
            item_id: "minecraft:diamond".into(),
            count: 2,
            dimension_id: "minecraft:overworld".into(),
            x: 1.5,
            y: 64.0,
            z: -3.25,
            age: 6_000,
        }
    }

    #[test]
    fn item_expire_attempt_uses_the_stable_schema_v1_wire_layout() {
        let event = sample();
        let encoded = to_bytes(&event).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_u32.to_le_bytes());
        for value in [&event.item_entity_id, &event.item_id] {
            expected.extend_from_slice(&(value.len() as u32).to_le_bytes());
            expected.extend_from_slice(value.as_bytes());
        }
        expected.extend_from_slice(&event.count.to_le_bytes());
        expected.extend_from_slice(&(event.dimension_id.len() as u32).to_le_bytes());
        expected.extend_from_slice(event.dimension_id.as_bytes());
        for value in [event.x, event.y, event.z] {
            expected.extend_from_slice(&value.to_bits().to_le_bytes());
        }
        expected.extend_from_slice(&event.age.to_le_bytes());
        assert_eq!(encoded, expected);
        assert_eq!(from_bytes::<ItemExpireAttempt>(&encoded).unwrap(), event);
    }

    #[test]
    fn item_expire_attempt_round_trips_negative_positions() {
        let mut event = sample();
        event.x = -29_999_999.5;
        event.y = -64.0;
        event.z = 12_345.678;
        event.age = 0;
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(from_bytes::<ItemExpireAttempt>(&encoded).unwrap(), event);
    }

    #[test]
    fn item_expire_attempt_rejects_other_schema_versions() {
        let mut encoded = to_bytes(&sample()).unwrap();
        encoded[..4].copy_from_slice(&2_u32.to_le_bytes());
        assert_eq!(
            from_bytes::<ItemExpireAttempt>(&encoded)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );
    }

    #[test]
    fn item_expire_attempt_constants_match_the_host_contract() {
        assert_eq!(super::ITEM_EXPIRE_ATTEMPT_EVENT_NAME, "item.expire_attempt");
        assert_eq!(
            super::ITEM_EXPIRE_ATTEMPT_CANCEL_CAPABILITY,
            "events.cancel"
        );
        assert_eq!(super::MAX_ITEM_EXPIRE_ATTEMPT_ID_BYTES, 256);
        assert_eq!(super::MAX_ITEM_EXPIRE_ATTEMPT_ITEM_COUNT, 1_000_000);
        assert_eq!(super::MAX_ITEM_EXPIRE_ATTEMPT_AGE, 1_000_000);
        assert_eq!(super::MAX_ITEM_EXPIRE_ATTEMPT_PAYLOAD_BYTES, 1_024);
        assert_eq!(super::MAX_ITEM_EXPIRE_ATTEMPT_CALLBACKS, 64);
        assert_eq!(super::MAX_ITEM_EXPIRE_ATTEMPT_COORDINATE, 30_000_000.0);
    }
}
