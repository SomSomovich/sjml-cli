//! Detached payload for cancellable logical-server game-mode changes.
//!
//! Register a `player.game_mode_change_attempt` callback with
//! `events.subscribe` and `events.cancel`. Returning `Ok(true)` rejects the
//! requested game-mode transition, so the player keeps their current mode. The
//! callback is synchronous because NeoForge checks the cancellation while it is
//! still dispatching the change event.

use crate::{StableDecode, StableEncode};

/// Event name subscribed with `events::subscribe`.
pub const PLAYER_GAME_MODE_CHANGE_ATTEMPT_EVENT_NAME: &str = "player.game_mode_change_attempt";
/// Cancellation grant required to reject game-mode changes; `events.subscribe` is also required.
pub const PLAYER_GAME_MODE_CHANGE_ATTEMPT_CANCEL_CAPABILITY: &str = "events.cancel";
/// Maximum UTF-8 byte length of one detached string in a game-mode callback.
pub const MAX_PLAYER_GAME_MODE_CHANGE_ATTEMPT_STRING_BYTES: usize = 256;
/// Maximum serialized payload size for one game-mode callback.
pub const MAX_PLAYER_GAME_MODE_CHANGE_ATTEMPT_PAYLOAD_BYTES: usize = 768;
/// Maximum callbacks invoked for one game-mode change attempt.
pub const MAX_PLAYER_GAME_MODE_CHANGE_ATTEMPT_CALLBACKS: usize = 64;

/// Synchronous cancellable snapshot captured when a player changes game mode.
///
/// NeoForge fires `player.game_mode_change_attempt` before the new game mode is
/// applied. Return `Ok(true)` to cancel the change. The detached snapshot
/// contains the player UUID, the player's dimension, and both the current and
/// requested game modes. Game-mode names are lowercase vanilla values:
/// `survival`, `creative`, `adventure`, or `spectator`.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerGameModeChangeAttempt {
    /// Canonical UUID of the player whose game mode would change.
    pub player_id: String,
    /// Resource location of the dimension containing the player.
    pub dimension_id: String,
    /// Current lowercase game-mode name before the change.
    pub current_game_mode: String,
    /// Requested lowercase game-mode name.
    pub new_game_mode: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_bytes, to_bytes, ErrorCode};

    fn sample() -> PlayerGameModeChangeAttempt {
        PlayerGameModeChangeAttempt {
            player_id: "00000000-0000-0000-0000-000000000001".into(),
            dimension_id: "minecraft:overworld".into(),
            current_game_mode: "survival".into(),
            new_game_mode: "creative".into(),
        }
    }

    fn expected_wire(event: &PlayerGameModeChangeAttempt) -> Vec<u8> {
        let mut output = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut output, &event.player_id);
        append_test_string(&mut output, &event.dimension_id);
        append_test_string(&mut output, &event.current_game_mode);
        append_test_string(&mut output, &event.new_game_mode);
        output
    }

    #[test]
    fn game_mode_change_attempt_matches_the_java_wire_layout_and_round_trips() {
        let event = sample();
        let expected = expected_wire(&event);
        assert_eq!(to_bytes(&event).unwrap(), expected);
        assert_eq!(
            from_bytes::<PlayerGameModeChangeAttempt>(&expected).unwrap(),
            event
        );
    }

    #[test]
    fn game_mode_change_attempt_round_trips_every_vanilla_mode_name() {
        for mode in ["survival", "creative", "adventure", "spectator"] {
            let mut event = sample();
            event.current_game_mode = mode.to_owned();
            event.new_game_mode = mode.to_owned();
            let encoded = to_bytes(&event).unwrap();
            assert_eq!(
                from_bytes::<PlayerGameModeChangeAttempt>(&encoded).unwrap(),
                event
            );
        }
    }

    #[test]
    fn game_mode_change_attempt_rejects_other_schema_versions() {
        let mut encoded = to_bytes(&sample()).unwrap();
        encoded[..4].copy_from_slice(&2_i32.to_le_bytes());
        assert_eq!(
            from_bytes::<PlayerGameModeChangeAttempt>(&encoded)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );
    }

    #[test]
    fn game_mode_change_attempt_constants_match_the_host_contract() {
        assert_eq!(
            PLAYER_GAME_MODE_CHANGE_ATTEMPT_EVENT_NAME,
            "player.game_mode_change_attempt"
        );
        assert_eq!(
            PLAYER_GAME_MODE_CHANGE_ATTEMPT_CANCEL_CAPABILITY,
            "events.cancel"
        );
        assert_eq!(MAX_PLAYER_GAME_MODE_CHANGE_ATTEMPT_STRING_BYTES, 256);
        assert_eq!(MAX_PLAYER_GAME_MODE_CHANGE_ATTEMPT_PAYLOAD_BYTES, 768);
        assert_eq!(MAX_PLAYER_GAME_MODE_CHANGE_ATTEMPT_CALLBACKS, 64);
    }

    fn append_test_string(output: &mut Vec<u8>, value: &str) {
        output.extend_from_slice(&(value.len() as i32).to_le_bytes());
        output.extend_from_slice(value.as_bytes());
    }
}
