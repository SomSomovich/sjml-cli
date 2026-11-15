//! Detached payload for cancellable logical-server permission-level changes.
//!
//! Register a `player.permissions_change_attempt` callback with
//! `events.subscribe` and `events.cancel`. Returning `Ok(true)` rejects the
//! operator promotion or demotion, so the player keeps their current permission
//! level. The callback is synchronous because NeoForge checks the cancellation
//! while it is still dispatching the change event.

use crate::{StableDecode, StableEncode};

/// Event name subscribed with `events::subscribe`.
pub const PLAYER_PERMISSIONS_CHANGE_ATTEMPT_EVENT_NAME: &str = "player.permissions_change_attempt";
/// Cancellation grant required to reject permission-level changes; `events.subscribe` is also required.
pub const PLAYER_PERMISSIONS_CHANGE_ATTEMPT_CANCEL_CAPABILITY: &str = "events.cancel";
/// Maximum UTF-8 byte length of one detached ID in a permission-change callback.
pub const MAX_PLAYER_PERMISSIONS_CHANGE_ATTEMPT_ID_BYTES: usize = 256;
/// Lowest permission level accepted in a permission-change callback.
pub const MIN_PLAYER_PERMISSIONS_CHANGE_ATTEMPT_LEVEL: u32 = 0;
/// Highest permission level accepted in a permission-change callback.
pub const MAX_PLAYER_PERMISSIONS_CHANGE_ATTEMPT_LEVEL: u32 = 4;
/// Maximum serialized payload size for one permission-change callback.
pub const MAX_PLAYER_PERMISSIONS_CHANGE_ATTEMPT_PAYLOAD_BYTES: usize = 512;
/// Maximum callbacks invoked for one permission-level change attempt.
pub const MAX_PLAYER_PERMISSIONS_CHANGE_ATTEMPT_CALLBACKS: usize = 64;

/// Synchronous cancellable snapshot captured when a player is opped or deopped.
///
/// NeoForge fires `player.permissions_change_attempt` before the new permission
/// level is applied. Return `Ok(true)` to cancel the change. The detached
/// snapshot contains the player UUID, the player's dimension, and the requested
/// new level together with the player's current old level. Both levels are
/// bounded to `0..=4`.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerPermissionsChangeAttempt {
    /// Canonical UUID of the player whose permission level would change.
    pub player_id: String,
    /// Resource location of the dimension containing the player.
    pub dimension_id: String,
    /// Requested permission level, bounded by the host to `0..=4`.
    pub new_level: u32,
    /// Current permission level before the change, bounded by the host to `0..=4`.
    pub old_level: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_bytes, to_bytes, ErrorCode};

    fn sample() -> PlayerPermissionsChangeAttempt {
        PlayerPermissionsChangeAttempt {
            player_id: "00000000-0000-0000-0000-000000000001".into(),
            dimension_id: "minecraft:overworld".into(),
            new_level: 4,
            old_level: 2,
        }
    }

    fn expected_wire(event: &PlayerPermissionsChangeAttempt) -> Vec<u8> {
        let mut output = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut output, &event.player_id);
        append_test_string(&mut output, &event.dimension_id);
        output.extend_from_slice(&event.new_level.to_le_bytes());
        output.extend_from_slice(&event.old_level.to_le_bytes());
        output
    }

    #[test]
    fn permissions_change_attempt_matches_the_java_wire_layout_and_round_trips() {
        let event = sample();
        let expected = expected_wire(&event);
        assert_eq!(to_bytes(&event).unwrap(), expected);
        assert_eq!(
            from_bytes::<PlayerPermissionsChangeAttempt>(&expected).unwrap(),
            event
        );
    }

    #[test]
    fn permissions_change_attempt_round_trips_level_bounds() {
        let mut event = sample();
        event.new_level = MIN_PLAYER_PERMISSIONS_CHANGE_ATTEMPT_LEVEL;
        event.old_level = MAX_PLAYER_PERMISSIONS_CHANGE_ATTEMPT_LEVEL;
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(
            from_bytes::<PlayerPermissionsChangeAttempt>(&encoded).unwrap(),
            event
        );
    }

    #[test]
    fn permissions_change_attempt_rejects_other_schema_versions() {
        let mut encoded = to_bytes(&sample()).unwrap();
        encoded[..4].copy_from_slice(&2_i32.to_le_bytes());
        assert_eq!(
            from_bytes::<PlayerPermissionsChangeAttempt>(&encoded)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );
    }

    #[test]
    fn permissions_change_attempt_constants_match_the_host_contract() {
        assert_eq!(
            PLAYER_PERMISSIONS_CHANGE_ATTEMPT_EVENT_NAME,
            "player.permissions_change_attempt"
        );
        assert_eq!(
            PLAYER_PERMISSIONS_CHANGE_ATTEMPT_CANCEL_CAPABILITY,
            "events.cancel"
        );
        assert_eq!(MAX_PLAYER_PERMISSIONS_CHANGE_ATTEMPT_ID_BYTES, 256);
        assert_eq!(MIN_PLAYER_PERMISSIONS_CHANGE_ATTEMPT_LEVEL, 0);
        assert_eq!(MAX_PLAYER_PERMISSIONS_CHANGE_ATTEMPT_LEVEL, 4);
        assert_eq!(MAX_PLAYER_PERMISSIONS_CHANGE_ATTEMPT_PAYLOAD_BYTES, 512);
        assert_eq!(MAX_PLAYER_PERMISSIONS_CHANGE_ATTEMPT_CALLBACKS, 64);
    }

    fn append_test_string(output: &mut Vec<u8>, value: &str) {
        output.extend_from_slice(&(value.len() as i32).to_le_bytes());
        output.extend_from_slice(value.as_bytes());
    }
}
