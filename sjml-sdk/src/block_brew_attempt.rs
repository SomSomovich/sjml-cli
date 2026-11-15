//! Detached payload for cancellable logical-server brewing attempts.
//!
//! Register a `block.brew_attempt` callback with `events.subscribe` and
//! `events.cancel`. The callback is synchronous because NeoForge applies the
//! brewing result immediately after the event returns.

use crate::{StableDecode, StableEncode};

/// Event name subscribed with `events::subscribe`.
pub const BLOCK_BREW_ATTEMPT_EVENT_NAME: &str = "block.brew_attempt";
/// Cancellation grant required to veto brewing attempts; `events.subscribe` is also required.
pub const BLOCK_BREW_ATTEMPT_CANCEL_CAPABILITY: &str = "events.cancel";
/// Number of brewing slots carried by the event: three bottles and one ingredient.
pub const MAX_BLOCK_BREW_ATTEMPT_STACKS: usize = 4;
/// Maximum UTF-8 byte length of one item resource ID.
pub const MAX_BLOCK_BREW_ATTEMPT_STRING_BYTES: usize = 256;
/// Maximum stack count accepted in one brewing slot.
pub const MAX_BLOCK_BREW_ATTEMPT_ITEM_COUNT: u32 = 1_000_000;
/// Maximum number of subscribed `block.brew_attempt` handlers per attempt.
pub const MAX_BLOCK_BREW_ATTEMPT_CALLBACKS: usize = 64;
/// Maximum encoded payload size in bytes.
pub const MAX_BLOCK_BREW_ATTEMPT_PAYLOAD_BYTES: usize = 2_048;

/// Synchronous cancellable snapshot of the four brewing slots.
///
/// NeoForge fires `block.brew_attempt` before vanilla brewing consumes the
/// ingredient. Return `Ok(true)` to cancel the whole brewing operation. The
/// payload carries only item identities and counts: the host leaves the event's
/// item array unchanged, so a callback can neither create nor duplicate items.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct BlockBrewAttempt {
    /// Detached slots in vanilla brewing-stand order; empty slots are `minecraft:air` with count 0.
    pub stacks: Vec<BrewItemSnapshot>,
}

/// Detached registry item and bounded count from one brewing slot.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct BrewItemSnapshot {
    /// Canonical registry ID of the item, or `minecraft:air` for an empty slot.
    pub item_id: String,
    /// Stack count, bounded by the host to `0..=1_000_000`; empty slots use 0.
    pub count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_bytes, to_bytes, ErrorCode};

    fn event() -> BlockBrewAttempt {
        BlockBrewAttempt {
            stacks: vec![
                BrewItemSnapshot {
                    item_id: "minecraft:potion".into(),
                    count: 3,
                },
                BrewItemSnapshot {
                    item_id: "minecraft:air".into(),
                    count: 0,
                },
                BrewItemSnapshot {
                    item_id: "minecraft:potion".into(),
                    count: 1,
                },
                BrewItemSnapshot {
                    item_id: "minecraft:nether_wart".into(),
                    count: 7,
                },
            ],
        }
    }

    #[test]
    fn event_matches_the_java_schema_v1_wire_bytes() {
        let encoded = to_bytes(&event()).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_i32.to_le_bytes());
        expected.extend_from_slice(&4_u32.to_le_bytes());
        for (item_id, count) in [
            ("minecraft:potion", 3_u32),
            ("minecraft:air", 0),
            ("minecraft:potion", 1),
            ("minecraft:nether_wart", 7),
        ] {
            expected.extend_from_slice(&1_i32.to_le_bytes());
            expected.extend_from_slice(&(item_id.len() as u32).to_le_bytes());
            expected.extend_from_slice(item_id.as_bytes());
            expected.extend_from_slice(&count.to_le_bytes());
        }
        assert_eq!(encoded, expected);
        assert_eq!(from_bytes::<BlockBrewAttempt>(&encoded).unwrap(), event());
    }

    #[test]
    fn empty_stack_list_round_trips() {
        let event = BlockBrewAttempt { stacks: Vec::new() };
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(&encoded[..4], &1_i32.to_le_bytes());
        assert_eq!(&encoded[4..], &0_u32.to_le_bytes());
        assert_eq!(from_bytes::<BlockBrewAttempt>(&encoded).unwrap(), event);
    }

    #[test]
    fn unsupported_schema_versions_are_rejected() {
        let mut encoded = to_bytes(&event()).unwrap();
        encoded[..4].copy_from_slice(&2_u32.to_le_bytes());
        assert_eq!(
            from_bytes::<BlockBrewAttempt>(&encoded).unwrap_err().code(),
            ErrorCode::IncompatibleAbiVersion
        );
    }
}
