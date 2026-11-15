//! Detached payload for cancellable logical-server grindstone output takes.
//!
//! Register a `player.grindstone_take_attempt` callback with `events.subscribe`
//! and `events.cancel`. The callback is synchronous because NeoForge applies the
//! take immediately after the event returns.

use crate::{StableDecode, StableEncode};

/// Event name subscribed with `events::subscribe`.
pub const PLAYER_GRINDSTONE_TAKE_ATTEMPT_EVENT_NAME: &str = "player.grindstone_take_attempt";
/// Cancellation grant required to veto grindstone takes; `events.subscribe` is also required.
pub const PLAYER_GRINDSTONE_TAKE_ATTEMPT_CANCEL_CAPABILITY: &str = "events.cancel";
/// Maximum UTF-8 byte length of one detached player, dimension, or item ID.
pub const MAX_PLAYER_GRINDSTONE_TAKE_ATTEMPT_STRING_BYTES: usize = 256;
/// Maximum stack count accepted in one grindstone item snapshot.
pub const MAX_PLAYER_GRINDSTONE_TAKE_ATTEMPT_ITEM_COUNT: u32 = 1_000_000;
/// Maximum experience amount accepted by one grindstone take snapshot.
pub const MAX_PLAYER_GRINDSTONE_TAKE_ATTEMPT_XP: u32 = 1_000;
/// Maximum number of subscribed `player.grindstone_take_attempt` handlers per attempt.
pub const MAX_PLAYER_GRINDSTONE_TAKE_ATTEMPT_CALLBACKS: usize = 64;
/// Maximum encoded payload size in bytes.
pub const MAX_PLAYER_GRINDSTONE_TAKE_ATTEMPT_PAYLOAD_BYTES: usize = 4_096;

/// Synchronous cancellable snapshot of one grindstone output take.
///
/// NeoForge fires `player.grindstone_take_attempt` from
/// `GrindstoneEvent.OnTakeItem` before vanilla consumes the inputs. Return
/// `Ok(true)` to cancel the take: the output is not taken, neither input is
/// consumed, and no experience is awarded. The payload carries only item
/// identities and counts, so a callback can neither create nor duplicate items.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerGrindstoneTakeAttempt {
    /// Canonical UUID of the player taking the output.
    pub player_id: String,
    /// Resource location of the dimension containing the grindstone.
    pub dimension_id: String,
    /// Current top input stack.
    pub top: GrindstoneItemSnapshot,
    /// Current bottom input stack.
    pub bottom: GrindstoneItemSnapshot,
    /// Top input stack that would remain after the take.
    pub new_top: GrindstoneItemSnapshot,
    /// Bottom input stack that would remain after the take.
    pub new_bottom: GrindstoneItemSnapshot,
    /// Experience awarded by the take, bounded by the host to `0..=1_000`.
    pub xp: u32,
}

/// Detached registry item and bounded count from one grindstone slot.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct GrindstoneItemSnapshot {
    /// Canonical registry ID of the item, or `minecraft:air` for an empty slot.
    pub item_id: String,
    /// Stack count, bounded by the host to `0..=1_000_000`; empty slots use 0.
    pub count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_bytes, to_bytes, ErrorCode};

    fn event() -> PlayerGrindstoneTakeAttempt {
        PlayerGrindstoneTakeAttempt {
            player_id: "00000000-0000-0000-0000-000000000001".into(),
            dimension_id: "minecraft:overworld".into(),
            top: GrindstoneItemSnapshot {
                item_id: "minecraft:diamond_sword".into(),
                count: 1,
            },
            bottom: GrindstoneItemSnapshot {
                item_id: "minecraft:diorite".into(),
                count: 1,
            },
            new_top: GrindstoneItemSnapshot {
                item_id: "minecraft:iron_sword".into(),
                count: 1,
            },
            new_bottom: GrindstoneItemSnapshot {
                item_id: "minecraft:air".into(),
                count: 0,
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
        for value in [
            "00000000-0000-0000-0000-000000000001",
            "minecraft:overworld",
        ] {
            expected.extend_from_slice(&(value.len() as u32).to_le_bytes());
            expected.extend_from_slice(value.as_bytes());
        }
        write_item(&mut expected, "minecraft:diamond_sword", 1);
        write_item(&mut expected, "minecraft:diorite", 1);
        write_item(&mut expected, "minecraft:iron_sword", 1);
        write_item(&mut expected, "minecraft:air", 0);
        expected.extend_from_slice(&5_u32.to_le_bytes());
        assert_eq!(encoded, expected);
        assert_eq!(
            from_bytes::<PlayerGrindstoneTakeAttempt>(&encoded).unwrap(),
            event()
        );
    }

    #[test]
    fn empty_grindstone_snapshot_round_trips() {
        let empty = GrindstoneItemSnapshot {
            item_id: "minecraft:air".into(),
            count: 0,
        };
        let event = PlayerGrindstoneTakeAttempt {
            player_id: "00000000-0000-0000-0000-000000000001".into(),
            dimension_id: "minecraft:the_nether".into(),
            top: empty.clone(),
            bottom: empty.clone(),
            new_top: empty.clone(),
            new_bottom: empty,
            xp: 0,
        };
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(&encoded[..4], &1_i32.to_le_bytes());
        assert_eq!(
            from_bytes::<PlayerGrindstoneTakeAttempt>(&encoded).unwrap(),
            event
        );
    }

    #[test]
    fn unsupported_schema_versions_are_rejected() {
        let mut encoded = to_bytes(&event()).unwrap();
        encoded[..4].copy_from_slice(&2_u32.to_le_bytes());
        assert_eq!(
            from_bytes::<PlayerGrindstoneTakeAttempt>(&encoded)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );
    }
}
