//! Detached payload for cancellable logical-server grindstone input placements.
//!
//! Register a `player.grindstone_place_attempt` callback with `events.subscribe`
//! and `events.cancel`. The callback is synchronous because NeoForge applies the
//! placement result immediately after the event returns.

use crate::{StableDecode, StableEncode};

/// Event name subscribed with `events::subscribe`.
pub const PLAYER_GRINDSTONE_PLACE_ATTEMPT_EVENT_NAME: &str = "player.grindstone_place_attempt";
/// Cancellation grant required to veto grindstone placements; `events.subscribe` is also required.
pub const PLAYER_GRINDSTONE_PLACE_ATTEMPT_CANCEL_CAPABILITY: &str = "events.cancel";
/// Maximum UTF-8 byte length of one item resource ID.
pub const MAX_PLAYER_GRINDSTONE_PLACE_ATTEMPT_STRING_BYTES: usize = 256;
/// Maximum stack count accepted in one grindstone item snapshot.
pub const MAX_PLAYER_GRINDSTONE_PLACE_ATTEMPT_ITEM_COUNT: u32 = 1_000_000;
/// Maximum experience amount accepted by one grindstone placement snapshot.
pub const MAX_PLAYER_GRINDSTONE_PLACE_ATTEMPT_XP: u32 = 1_000;
/// Maximum number of subscribed `player.grindstone_place_attempt` handlers per attempt.
pub const MAX_PLAYER_GRINDSTONE_PLACE_ATTEMPT_CALLBACKS: usize = 64;
/// Maximum encoded payload size in bytes.
pub const MAX_PLAYER_GRINDSTONE_PLACE_ATTEMPT_PAYLOAD_BYTES: usize = 2_048;

/// Synchronous cancellable snapshot of one grindstone input placement.
///
/// NeoForge fires `player.grindstone_place_attempt` from
/// `GrindstoneEvent.OnPlaceItem` when the grindstone inputs change. Return
/// `Ok(true)` to cancel the placement: vanilla does not compute an output and
/// the output slot becomes empty. The payload carries only item identities and
/// counts, so a callback can neither create nor duplicate items.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerGrindstonePlaceAttempt {
    /// Current top input stack.
    pub top: GrindstonePlaceItemSnapshot,
    /// Current bottom input stack.
    pub bottom: GrindstonePlaceItemSnapshot,
    /// Experience amount supplied by the placement, bounded by the host to `0..=1_000`.
    pub xp: u32,
}

/// Detached registry item and bounded count from one grindstone slot.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct GrindstonePlaceItemSnapshot {
    /// Canonical registry ID of the item, or `minecraft:air` for an empty slot.
    pub item_id: String,
    /// Stack count, bounded by the host to `0..=1_000_000`; empty slots use 0.
    pub count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_bytes, to_bytes, ErrorCode};

    fn event() -> PlayerGrindstonePlaceAttempt {
        PlayerGrindstonePlaceAttempt {
            top: GrindstonePlaceItemSnapshot {
                item_id: "minecraft:diamond_sword".into(),
                count: 1,
            },
            bottom: GrindstonePlaceItemSnapshot {
                item_id: "minecraft:diorite".into(),
                count: 1,
            },
            xp: 5,
        }
    }

    fn write_item(expected: &mut Vec<u8>, item_id: &str, count: u32) {
        expected.extend_from_slice(&1_i32.to_le_bytes());
        expected.extend_from_slice(&(item_id.len() as u32).to_le_bytes());
        expected.extend_from_slice(item_id.as_bytes());
        expected.extend_from_slice(&count.to_le_bytes());
    }

    #[test]
    fn event_matches_the_java_schema_v1_wire_bytes() {
        let encoded = to_bytes(&event()).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_i32.to_le_bytes());
        write_item(&mut expected, "minecraft:diamond_sword", 1);
        write_item(&mut expected, "minecraft:diorite", 1);
        expected.extend_from_slice(&5_u32.to_le_bytes());
        assert_eq!(encoded, expected);
        assert_eq!(
            from_bytes::<PlayerGrindstonePlaceAttempt>(&encoded).unwrap(),
            event()
        );
    }

    #[test]
    fn empty_grindstone_snapshot_round_trips() {
        let empty = GrindstonePlaceItemSnapshot {
            item_id: "minecraft:air".into(),
            count: 0,
        };
        let event = PlayerGrindstonePlaceAttempt {
            top: empty.clone(),
            bottom: empty,
            xp: 0,
        };
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(&encoded[..4], &1_i32.to_le_bytes());
        assert_eq!(
            from_bytes::<PlayerGrindstonePlaceAttempt>(&encoded).unwrap(),
            event
        );
    }

    #[test]
    fn unsupported_schema_versions_are_rejected() {
        let mut encoded = to_bytes(&event()).unwrap();
        encoded[..4].copy_from_slice(&2_u32.to_le_bytes());
        assert_eq!(
            from_bytes::<PlayerGrindstonePlaceAttempt>(&encoded)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );
    }
}
