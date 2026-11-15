//! Synchronous bow-draw veto values for `player.arrow_nock_attempt`.
//!
//! NeoForge fires the event when a player begins drawing a bow or crossbow.
//! Return `Ok(true)` to cancel the use action, which makes the draw fail.
//! Register the callback with `events.subscribe` and grant `events.cancel`;
//! asynchronous handlers are rejected because the veto must be known before
//! the event returns.

use crate::{StableDecode, StableEncode};

/// Event name for a synchronous bow-draw veto.
pub const PLAYER_ARROW_NOCK_ATTEMPT_EVENT_NAME: &str = "player.arrow_nock_attempt";
/// Cancellation grant required to veto a bow draw; `events.subscribe` is also required.
pub const PLAYER_ARROW_NOCK_ATTEMPT_CANCEL_CAPABILITY: &str = "events.cancel";
/// Maximum UTF-8 byte length of one detached UUID, resource location, or hand name.
pub const MAX_PLAYER_ARROW_NOCK_ATTEMPT_STRING_BYTES: usize = 256;
/// Maximum stack count represented in a bow-draw callback.
pub const MAX_PLAYER_ARROW_NOCK_ATTEMPT_ITEM_COUNT: i32 = 1_000_000;
/// Maximum callbacks invoked for one bow-draw event.
pub const MAX_PLAYER_ARROW_NOCK_ATTEMPT_CALLBACKS: usize = 64;
/// Maximum schema-v1 payload size for one bow-draw callback.
pub const MAX_PLAYER_ARROW_NOCK_ATTEMPT_PAYLOAD_BYTES: usize = 1_024;

/// Detached logical-server values observed when a player begins drawing a bow.
///
/// All fields are detached strings, numbers, and flags; no `Player`, item
/// stack, level, or host handle crosses the guest ABI.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerArrowNockAttempt {
    /// Canonical UUID of the player beginning the draw.
    pub player_id: String,
    /// Resource location of the dimension containing the player.
    pub dimension_id: String,
    /// Lowercase hand name: `main_hand` or `off_hand`.
    pub hand: String,
    /// Canonical item ID of the held item, or `minecraft:air` when empty.
    pub item_id: String,
    /// Stack count of the held item, from 0 through 1,000,000.
    pub count: i32,
    /// Whether the player has ammo available for the bow.
    pub has_ammo: bool,
}

#[cfg(test)]
mod tests {
    use super::{
        PlayerArrowNockAttempt, MAX_PLAYER_ARROW_NOCK_ATTEMPT_CALLBACKS,
        MAX_PLAYER_ARROW_NOCK_ATTEMPT_ITEM_COUNT, MAX_PLAYER_ARROW_NOCK_ATTEMPT_PAYLOAD_BYTES,
        MAX_PLAYER_ARROW_NOCK_ATTEMPT_STRING_BYTES,
    };
    use crate::{from_bytes, to_bytes};

    fn sample() -> PlayerArrowNockAttempt {
        PlayerArrowNockAttempt {
            player_id: "00000000-0000-0000-0000-000000000001".into(),
            dimension_id: "minecraft:overworld".into(),
            hand: "main_hand".into(),
            item_id: "minecraft:bow".into(),
            count: 1,
            has_ammo: true,
        }
    }

    #[test]
    fn arrow_nock_attempt_matches_the_java_schema_v1_layout() {
        let event = sample();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_string(&mut expected, &event.player_id);
        append_string(&mut expected, &event.dimension_id);
        append_string(&mut expected, &event.hand);
        append_string(&mut expected, &event.item_id);
        expected.extend_from_slice(&event.count.to_le_bytes());
        expected.push(u8::from(event.has_ammo));
        assert_eq!(to_bytes(&event).unwrap(), expected);
        assert_eq!(
            from_bytes::<PlayerArrowNockAttempt>(&expected).unwrap(),
            event
        );
    }

    #[test]
    fn arrow_nock_attempt_round_trips_an_empty_off_hand() {
        let mut event = sample();
        event.hand = "off_hand".into();
        event.item_id = "minecraft:air".into();
        event.count = 0;
        event.has_ammo = false;
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(
            from_bytes::<PlayerArrowNockAttempt>(&encoded).unwrap(),
            event
        );
    }

    #[test]
    fn unsupported_schema_versions_are_rejected() {
        let mut encoded = to_bytes(&sample()).unwrap();
        encoded[..4].copy_from_slice(&2_u32.to_le_bytes());
        assert!(from_bytes::<PlayerArrowNockAttempt>(&encoded).is_err());
    }

    #[test]
    fn abi_bounds_match_the_host_contract() {
        assert_eq!(MAX_PLAYER_ARROW_NOCK_ATTEMPT_STRING_BYTES, 256);
        assert_eq!(MAX_PLAYER_ARROW_NOCK_ATTEMPT_ITEM_COUNT, 1_000_000);
        assert_eq!(MAX_PLAYER_ARROW_NOCK_ATTEMPT_CALLBACKS, 64);
        assert_eq!(MAX_PLAYER_ARROW_NOCK_ATTEMPT_PAYLOAD_BYTES, 1_024);
    }

    fn append_string(output: &mut Vec<u8>, value: &str) {
        output.extend_from_slice(&(value.len() as i32).to_le_bytes());
        output.extend_from_slice(value.as_bytes());
    }
}
