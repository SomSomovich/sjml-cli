//! Detached observation of server players repairing or renaming items in an anvil.
//!
//! Register a `player.anvil_repair` callback with `events.subscribe`. The
//! notification is observation-only and asynchronous handlers are allowed,
//! because the snapshot is detached from the NeoForge event.

use crate::{StableDecode, StableEncode};

/// Event name subscribed with `events::subscribe`.
pub const PLAYER_ANVIL_REPAIR_EVENT_NAME: &str = "player.anvil_repair";
/// Guest export the host invokes for one `player.anvil_repair` callback.
pub const PLAYER_ANVIL_REPAIR_EXPORT_NAME: &str = "sjml_v1_event_player.anvil_repair";
/// Maximum UTF-8 byte length of a player UUID, dimension, or item resource ID.
pub const MAX_PLAYER_ANVIL_REPAIR_STRING_BYTES: usize = 256;
/// Maximum stack count accepted in one anvil slot.
pub const MAX_PLAYER_ANVIL_REPAIR_ITEM_COUNT: u32 = 1_000_000;
/// Maximum number of subscribed `player.anvil_repair` handlers per event.
pub const MAX_PLAYER_ANVIL_REPAIR_CALLBACKS: usize = 64;
/// Maximum encoded payload size in bytes.
pub const MAX_PLAYER_ANVIL_REPAIR_PAYLOAD_BYTES: usize = 2_048;
/// Minimum accepted anvil break chance.
pub const MIN_PLAYER_ANVIL_REPAIR_BREAK_CHANCE: f32 = 0.0;
/// Maximum accepted anvil break chance.
pub const MAX_PLAYER_ANVIL_REPAIR_BREAK_CHANCE: f32 = 1.0;

/// Observation-only snapshot emitted after a server player takes an anvil result.
///
/// NeoForge fires `AnvilRepairEvent` when the player removes the repaired or
/// renamed item from the anvil's output slot. SJML dispatches the detached
/// snapshot only on the owning logical-server thread. The notification is not
/// cancellable, so a handler returns `()` rather than a veto. All values are
/// detached; no `Player`, `ItemStack`, or level reference crosses the guest ABI.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerAnvilRepair {
    /// Canonical UUID of the player who completed the anvil operation.
    pub player_id: String,
    /// Resource location of the player's dimension.
    pub dimension_id: String,
    /// Item taken from the anvil output slot, or `minecraft:air` with count 0.
    pub output: AnvilRepairItemSnapshot,
    /// Item from the left anvil input slot, or `minecraft:air` with count 0.
    pub left: AnvilRepairItemSnapshot,
    /// Item from the right anvil input slot, or `minecraft:air` with count 0.
    pub right: AnvilRepairItemSnapshot,
    /// Chance that the anvil is damaged, finite in the `0.0..=1.0` range.
    pub break_chance: f32,
}

/// Detached registry item and bounded count from one anvil slot.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct AnvilRepairItemSnapshot {
    /// Canonical registry ID of the item, or `minecraft:air` for an empty slot.
    pub item_id: String,
    /// Stack count, bounded by the host to `0..=1_000_000`; empty slots use 0.
    pub count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_bytes, to_bytes, ErrorCode};

    fn event() -> PlayerAnvilRepair {
        PlayerAnvilRepair {
            player_id: "00000000-0000-0000-0000-000000000001".into(),
            dimension_id: "minecraft:overworld".into(),
            output: AnvilRepairItemSnapshot {
                item_id: "minecraft:diamond_sword".into(),
                count: 1,
            },
            left: AnvilRepairItemSnapshot {
                item_id: "minecraft:diamond_sword".into(),
                count: 1,
            },
            right: AnvilRepairItemSnapshot {
                item_id: "minecraft:air".into(),
                count: 0,
            },
            break_chance: 0.12,
        }
    }

    fn expected_wire(event: &PlayerAnvilRepair) -> Vec<u8> {
        let mut output = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut output, &event.player_id);
        append_test_string(&mut output, &event.dimension_id);
        append_test_item(&mut output, &event.output);
        append_test_item(&mut output, &event.left);
        append_test_item(&mut output, &event.right);
        output.extend_from_slice(&event.break_chance.to_bits().to_le_bytes());
        output
    }

    #[test]
    fn anvil_repair_matches_the_java_wire_layout_and_round_trips() {
        let event = event();
        let expected = expected_wire(&event);
        assert_eq!(to_bytes(&event).unwrap(), expected);
        assert_eq!(from_bytes::<PlayerAnvilRepair>(&expected).unwrap(), event);
    }

    #[test]
    fn empty_slots_keep_their_nested_schema_and_round_trip() {
        let event = PlayerAnvilRepair {
            output: AnvilRepairItemSnapshot {
                item_id: "minecraft:air".into(),
                count: 0,
            },
            left: AnvilRepairItemSnapshot {
                item_id: "minecraft:air".into(),
                count: 0,
            },
            right: AnvilRepairItemSnapshot {
                item_id: "minecraft:diamond".into(),
                count: 1,
            },
            ..event()
        };
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(encoded, expected_wire(&event));
        assert_eq!(from_bytes::<PlayerAnvilRepair>(&encoded).unwrap(), event);
    }

    #[test]
    fn unsupported_schema_versions_are_rejected() {
        let mut encoded = to_bytes(&event()).unwrap();
        encoded[..4].copy_from_slice(&2_i32.to_le_bytes());
        assert_eq!(
            from_bytes::<PlayerAnvilRepair>(&encoded)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );
    }

    fn append_test_string(output: &mut Vec<u8>, value: &str) {
        output.extend_from_slice(&(value.len() as i32).to_le_bytes());
        output.extend_from_slice(value.as_bytes());
    }

    fn append_test_item(output: &mut Vec<u8>, item: &AnvilRepairItemSnapshot) {
        output.extend_from_slice(&1_i32.to_le_bytes());
        append_test_string(output, &item.item_id);
        output.extend_from_slice(&item.count.to_le_bytes());
    }
}
