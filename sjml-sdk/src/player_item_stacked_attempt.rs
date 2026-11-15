//! Detached payload for a cancellable logical-server slot-stacking attempt.
//!
//! Register a `player.item_stacked_attempt` callback with `events.subscribe`
//! and `events.cancel`. The callback is synchronous because NeoForge applies
//! the carried-item interaction immediately after the event returns.

use crate::{StableDecode, StableEncode};

/// Event name subscribed with `events::subscribe`.
pub const PLAYER_ITEM_STACKED_ATTEMPT_EVENT_NAME: &str = "player.item_stacked_attempt";
/// Cancellation grant required to veto slot stacking; `events.subscribe` is also required.
pub const PLAYER_ITEM_STACKED_ATTEMPT_CANCEL_CAPABILITY: &str = "events.cancel";
/// Maximum UTF-8 byte length of a player, dimension, item, or click-action string.
pub const MAX_PLAYER_ITEM_STACKED_ATTEMPT_STRING_BYTES: usize = 256;
/// Maximum stack count accepted for one carried or destination snapshot.
pub const MAX_PLAYER_ITEM_STACKED_ATTEMPT_ITEM_COUNT: u32 = 1_000_000;
/// Maximum lowercase click-action length in bytes.
pub const MAX_PLAYER_ITEM_STACKED_ATTEMPT_CLICK_ACTION_BYTES: usize = 32;
/// Maximum slot index accepted for one stacking attempt.
pub const MAX_PLAYER_ITEM_STACKED_ATTEMPT_SLOT_INDEX: i32 = 255;
/// Maximum number of subscribed `player.item_stacked_attempt` handlers per click.
pub const MAX_PLAYER_ITEM_STACKED_ATTEMPT_CALLBACKS: usize = 64;
/// Maximum encoded payload size in bytes.
pub const MAX_PLAYER_ITEM_STACKED_ATTEMPT_PAYLOAD_BYTES: usize = 2_048;

/// Synchronous cancellable snapshot of a carried stack clicked onto a slot.
///
/// NeoForge fires `player.item_stacked_attempt` when a player clicks a slot
/// with a carried item stack that would stack onto the slot contents, for
/// example while moving items in a container. Return `Ok(true)` to reject the
/// stacking interaction, which leaves both stacks unchanged. The payload
/// carries only item identities and counts: callbacks cannot change either
/// stack or the slot.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerItemStackedAttempt {
    /// Canonical UUID of the player performing the stacking attempt.
    pub player_id: String,
    /// Resource location of the dimension containing the player.
    pub dimension_id: String,
    /// Stack carried by the cursor when the slot was clicked.
    pub carried: StackedItemSnapshot,
    /// Stack currently in the clicked slot.
    pub stacked_on: StackedItemSnapshot,
    /// Lowercase click action, such as `pickup` or `quick_move`.
    pub click_action: String,
    /// Index of the clicked slot, bounded by the host to `0..=255`.
    pub slot_index: i32,
}

/// Detached registry item and bounded count from one carried or slot stack.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct StackedItemSnapshot {
    /// Canonical registry ID of the item, or `minecraft:air` for an empty stack.
    pub item_id: String,
    /// Stack count, bounded by the host to `0..=1_000_000`; empty stacks use 0.
    pub count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_bytes, to_bytes, ErrorCode};

    fn event() -> PlayerItemStackedAttempt {
        PlayerItemStackedAttempt {
            player_id: "00000000-0000-0000-0000-000000000001".into(),
            dimension_id: "minecraft:overworld".into(),
            carried: StackedItemSnapshot {
                item_id: "minecraft:diamond".into(),
                count: 3,
            },
            stacked_on: StackedItemSnapshot {
                item_id: "minecraft:dirt".into(),
                count: 1,
            },
            click_action: "pickup".into(),
            slot_index: 5,
        }
    }

    #[test]
    fn event_matches_the_java_schema_v1_wire_bytes() {
        let encoded = to_bytes(&event()).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_i32.to_le_bytes());
        for value in [
            "00000000-0000-0000-0000-000000000001",
            "minecraft:overworld",
        ] {
            expected.extend_from_slice(&(value.len() as u32).to_le_bytes());
            expected.extend_from_slice(value.as_bytes());
        }
        for (item_id, count) in [("minecraft:diamond", 3_u32), ("minecraft:dirt", 1)] {
            expected.extend_from_slice(&1_i32.to_le_bytes());
            expected.extend_from_slice(&(item_id.len() as u32).to_le_bytes());
            expected.extend_from_slice(item_id.as_bytes());
            expected.extend_from_slice(&count.to_le_bytes());
        }
        let sample = event();
        expected.extend_from_slice(&(sample.click_action.len() as u32).to_le_bytes());
        expected.extend_from_slice(sample.click_action.as_bytes());
        expected.extend_from_slice(&sample.slot_index.to_le_bytes());
        assert_eq!(encoded, expected);
        assert_eq!(
            from_bytes::<PlayerItemStackedAttempt>(&encoded).unwrap(),
            event()
        );
    }

    #[test]
    fn empty_destination_round_trips_as_air() {
        let mut event = event();
        event.stacked_on = StackedItemSnapshot {
            item_id: "minecraft:air".into(),
            count: 0,
        };
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(
            from_bytes::<PlayerItemStackedAttempt>(&encoded).unwrap(),
            event
        );
    }

    #[test]
    fn slot_and_click_action_round_trip_at_their_bounds() {
        let mut event = event();
        event.click_action = "quick_craft".into();
        event.slot_index = MAX_PLAYER_ITEM_STACKED_ATTEMPT_SLOT_INDEX;
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(
            from_bytes::<PlayerItemStackedAttempt>(&encoded).unwrap(),
            event
        );
    }

    #[test]
    fn unsupported_schema_versions_are_rejected() {
        let mut encoded = to_bytes(&event()).unwrap();
        encoded[..4].copy_from_slice(&2_i32.to_le_bytes());
        assert_eq!(
            from_bytes::<PlayerItemStackedAttempt>(&encoded)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );

        let event = event();
        let nested_schema = 4 + (4 + event.player_id.len()) + (4 + event.dimension_id.len());
        let mut encoded = to_bytes(&event).unwrap();
        encoded[nested_schema..nested_schema + 4].copy_from_slice(&2_i32.to_le_bytes());
        assert_eq!(
            from_bytes::<PlayerItemStackedAttempt>(&encoded)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );
    }

    #[test]
    fn constants_match_the_host_contract() {
        assert_eq!(
            PLAYER_ITEM_STACKED_ATTEMPT_EVENT_NAME,
            "player.item_stacked_attempt"
        );
        assert_eq!(
            PLAYER_ITEM_STACKED_ATTEMPT_CANCEL_CAPABILITY,
            "events.cancel"
        );
        assert_eq!(MAX_PLAYER_ITEM_STACKED_ATTEMPT_STRING_BYTES, 256);
        assert_eq!(MAX_PLAYER_ITEM_STACKED_ATTEMPT_ITEM_COUNT, 1_000_000);
        assert_eq!(MAX_PLAYER_ITEM_STACKED_ATTEMPT_CLICK_ACTION_BYTES, 32);
        assert_eq!(MAX_PLAYER_ITEM_STACKED_ATTEMPT_SLOT_INDEX, 255);
        assert_eq!(MAX_PLAYER_ITEM_STACKED_ATTEMPT_CALLBACKS, 64);
        assert_eq!(MAX_PLAYER_ITEM_STACKED_ATTEMPT_PAYLOAD_BYTES, 2_048);
    }
}
