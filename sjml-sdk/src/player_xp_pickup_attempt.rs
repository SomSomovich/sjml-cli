//! Detached payload for cancellable logical-server experience-orb pickups.
//!
//! Register a `player.xp_pickup_attempt` callback with `events.subscribe` and
//! `events.cancel`. Returning `Ok(true)` prevents the player from collecting
//! the orb. The callback is synchronous because NeoForge checks the cancellation
//! before the orb value is added to the player.

use crate::{StableDecode, StableEncode};

/// Event name subscribed with `events::subscribe`.
pub const PLAYER_XP_PICKUP_ATTEMPT_EVENT_NAME: &str = "player.xp_pickup_attempt";
/// Cancellation grant required to reject orb pickups; `events.subscribe` is also required.
pub const PLAYER_XP_PICKUP_ATTEMPT_CANCEL_CAPABILITY: &str = "events.cancel";
/// Maximum UTF-8 byte length of one detached ID in an orb-pickup callback.
pub const MAX_PLAYER_XP_PICKUP_ATTEMPT_STRING_BYTES: usize = 256;
/// Lowest experience-orb value accepted in an orb-pickup callback.
pub const MIN_PLAYER_XP_PICKUP_ATTEMPT_ORB_VALUE: i32 = 0;
/// Highest experience-orb value accepted in an orb-pickup callback.
pub const MAX_PLAYER_XP_PICKUP_ATTEMPT_ORB_VALUE: i32 = 1_000_000;
/// Maximum serialized payload size for one orb-pickup callback.
pub const MAX_PLAYER_XP_PICKUP_ATTEMPT_PAYLOAD_BYTES: usize = 768;
/// Maximum callbacks invoked for one experience-orb pickup attempt.
pub const MAX_PLAYER_XP_PICKUP_ATTEMPT_CALLBACKS: usize = 64;

/// Synchronous cancellable snapshot captured before a player collects an orb.
///
/// NeoForge fires `player.xp_pickup_attempt` before a player picks up an
/// experience orb. Return `Ok(true)` to cancel the pickup, which leaves the orb
/// in the world and grants no experience. The detached snapshot contains the
/// player UUID, the player's dimension, the orb UUID, and the orb value bounded
/// to `0..=1_000_000`.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerXpPickupAttempt {
    /// Canonical UUID of the player collecting the orb.
    pub player_id: String,
    /// Resource location of the dimension containing the player.
    pub dimension_id: String,
    /// Canonical UUID of the experience orb.
    pub orb_id: String,
    /// Experience value of the orb, bounded by the host to `0..=1_000_000`.
    pub orb_value: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_bytes, to_bytes, ErrorCode};

    fn event() -> PlayerXpPickupAttempt {
        PlayerXpPickupAttempt {
            player_id: "00000000-0000-0000-0000-000000000001".into(),
            dimension_id: "minecraft:overworld".into(),
            orb_id: "00000000-0000-0000-0000-000000000002".into(),
            orb_value: 7,
        }
    }

    fn expected_wire(event: &PlayerXpPickupAttempt) -> Vec<u8> {
        let mut output = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut output, &event.player_id);
        append_test_string(&mut output, &event.dimension_id);
        append_test_string(&mut output, &event.orb_id);
        output.extend_from_slice(&event.orb_value.to_le_bytes());
        output
    }

    #[test]
    fn xp_pickup_attempt_matches_the_java_wire_layout_and_round_trips() {
        let event = event();
        let expected = expected_wire(&event);
        assert_eq!(to_bytes(&event).unwrap(), expected);
        assert_eq!(
            from_bytes::<PlayerXpPickupAttempt>(&expected).unwrap(),
            event
        );
    }

    #[test]
    fn xp_pickup_attempt_round_trips_orb_value_bounds() {
        let mut event = event();
        event.orb_value = MIN_PLAYER_XP_PICKUP_ATTEMPT_ORB_VALUE;
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(
            from_bytes::<PlayerXpPickupAttempt>(&encoded).unwrap(),
            event
        );

        event.orb_value = MAX_PLAYER_XP_PICKUP_ATTEMPT_ORB_VALUE;
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(
            from_bytes::<PlayerXpPickupAttempt>(&encoded).unwrap(),
            event
        );
    }

    #[test]
    fn xp_pickup_attempt_rejects_other_schema_versions() {
        let mut encoded = to_bytes(&event()).unwrap();
        encoded[..4].copy_from_slice(&2_i32.to_le_bytes());
        assert_eq!(
            from_bytes::<PlayerXpPickupAttempt>(&encoded)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );
    }

    #[test]
    fn xp_pickup_attempt_constants_match_the_host_contract() {
        assert_eq!(
            PLAYER_XP_PICKUP_ATTEMPT_EVENT_NAME,
            "player.xp_pickup_attempt"
        );
        assert_eq!(PLAYER_XP_PICKUP_ATTEMPT_CANCEL_CAPABILITY, "events.cancel");
        assert_eq!(MAX_PLAYER_XP_PICKUP_ATTEMPT_STRING_BYTES, 256);
        assert_eq!(MIN_PLAYER_XP_PICKUP_ATTEMPT_ORB_VALUE, 0);
        assert_eq!(MAX_PLAYER_XP_PICKUP_ATTEMPT_ORB_VALUE, 1_000_000);
        assert_eq!(MAX_PLAYER_XP_PICKUP_ATTEMPT_PAYLOAD_BYTES, 768);
        assert_eq!(MAX_PLAYER_XP_PICKUP_ATTEMPT_CALLBACKS, 64);
    }

    fn append_test_string(output: &mut Vec<u8>, value: &str) {
        output.extend_from_slice(&(value.len() as i32).to_le_bytes());
        output.extend_from_slice(value.as_bytes());
    }
}
