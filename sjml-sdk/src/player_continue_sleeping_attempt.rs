//! Detached payload for `player.continue_sleeping_attempt`.
//!
//! Register a `player.continue_sleeping_attempt` callback with `events.subscribe`
//! and `events.cancel`. Returning `Ok(true)` calls
//! `CanContinueSleepingEvent#setContinueSleeping(false)`, so the sleeper is
//! forced to wake. The callback is synchronous because vanilla consumes the
//! result immediately after the event returns.

use crate::{StableDecode, StableEncode};

/// Event name used with [`crate::events::subscribe`].
pub const EVENT_NAME: &str = "player.continue_sleeping_attempt";
/// Generated ABI export name for the typed continue-sleeping callback.
pub const EXPORT_NAME: &str = "sjml_v1_event_player.continue_sleeping_attempt";
/// Maximum UTF-8 byte length of the player UUID, dimension ID, and problem.
pub const MAX_STRING_BYTES: usize = 256;
/// Maximum encoded bytes for one detached continue-sleeping event.
pub const MAX_PAYLOAD_BYTES: usize = 1_024;
/// Maximum guest callbacks invoked for one continue-sleeping event.
pub const MAX_CALLBACKS: usize = 64;
/// Cancellation grant required to veto continued sleeping; `events.subscribe` is also required.
pub const CANCEL_CAPABILITY: &str = "events.cancel";

/// Synchronous logical-server values observed while a sleeping entity is checked.
///
/// NeoForge fires `CanContinueSleepingEvent` for a sleeping entity on its owning
/// logical-server thread. The snapshot contains the sleeper UUID, the dimension
/// ID, the lowercase `BedSleepingProblem` name (empty when there is no problem),
/// and the current continued-sleeping result. No entity or level reference
/// crosses the guest ABI.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerContinueSleepingAttempt {
    /// Canonical UUID of the sleeping entity.
    pub player_id: String,
    /// Resource location of the dimension containing the sleeper.
    pub dimension_id: String,
    /// Lowercase sleep problem name, or an empty string when no problem is set.
    pub problem: String,
    /// Whether the sleeper may currently continue sleeping.
    pub may_continue_sleeping: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_bytes, to_bytes, ErrorCode};

    fn event() -> PlayerContinueSleepingAttempt {
        PlayerContinueSleepingAttempt {
            player_id: "00000000-0000-0000-0000-000000000001".into(),
            dimension_id: "minecraft:overworld".into(),
            problem: "not_possible_now".into(),
            may_continue_sleeping: false,
        }
    }

    fn expected_wire(event: &PlayerContinueSleepingAttempt) -> Vec<u8> {
        let mut output = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut output, &event.player_id);
        append_test_string(&mut output, &event.dimension_id);
        append_test_string(&mut output, &event.problem);
        output.push(u8::from(event.may_continue_sleeping));
        output
    }

    #[test]
    fn continue_sleeping_attempt_matches_the_java_wire_layout_and_round_trips() {
        let event = event();
        let expected = expected_wire(&event);
        assert_eq!(to_bytes(&event).unwrap(), expected);
        assert_eq!(
            from_bytes::<PlayerContinueSleepingAttempt>(&expected).unwrap(),
            event
        );
    }

    #[test]
    fn empty_problem_and_allowed_result_round_trip() {
        let event = PlayerContinueSleepingAttempt {
            problem: String::new(),
            may_continue_sleeping: true,
            ..event()
        };
        let expected = expected_wire(&event);
        assert_eq!(to_bytes(&event).unwrap(), expected);
        assert_eq!(
            from_bytes::<PlayerContinueSleepingAttempt>(&expected).unwrap(),
            event
        );
    }

    #[test]
    fn continue_sleeping_attempt_rejects_a_foreign_schema_and_boolean_bytes() {
        let mut wrong_version = to_bytes(&event()).unwrap();
        wrong_version[..4].copy_from_slice(&2_i32.to_le_bytes());
        assert_eq!(
            from_bytes::<PlayerContinueSleepingAttempt>(&wrong_version)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );

        let event = event();
        let mut wrong_boolean = to_bytes(&event).unwrap();
        let flag_offset = 4
            + (4 + event.player_id.len())
            + (4 + event.dimension_id.len())
            + (4 + event.problem.len());
        wrong_boolean[flag_offset] = 2;
        assert_eq!(
            from_bytes::<PlayerContinueSleepingAttempt>(&wrong_boolean)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidBoolean
        );
    }

    #[test]
    fn published_limits_match_the_host_contract() {
        assert_eq!("player.continue_sleeping_attempt", EVENT_NAME);
        assert_eq!(
            "sjml_v1_event_player.continue_sleeping_attempt",
            EXPORT_NAME
        );
        assert_eq!(256, MAX_STRING_BYTES);
        assert_eq!(1_024, MAX_PAYLOAD_BYTES);
        assert_eq!(64, MAX_CALLBACKS);
        assert_eq!("events.cancel", CANCEL_CAPABILITY);
    }

    fn append_test_string(output: &mut Vec<u8>, value: &str) {
        output.extend_from_slice(&(value.len() as i32).to_le_bytes());
        output.extend_from_slice(value.as_bytes());
    }
}
