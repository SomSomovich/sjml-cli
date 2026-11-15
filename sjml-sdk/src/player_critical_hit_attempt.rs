//! Detached payload for `player.critical_hit_attempt`.
//!
//! Register a `player.critical_hit_attempt` callback with `events.subscribe` and
//! `events.cancel`. Returning `Ok(true)` clears the event's critical-hit flag,
//! which keeps the attack but removes its critical damage. The callback is
//! synchronous because NeoForge applies the attack damage immediately after the
//! event returns.

use crate::{StableDecode, StableEncode};

/// Maximum UTF-8 byte length of one detached ID in a critical-hit callback.
pub const MAX_PLAYER_CRITICAL_HIT_ATTEMPT_ID_BYTES: usize = 256;
/// Maximum serialized payload size for one `player.critical_hit_attempt` callback.
pub const MAX_PLAYER_CRITICAL_HIT_ATTEMPT_PAYLOAD_BYTES: usize = 2_048;
/// Maximum callbacks invoked for one critical-hit attempt.
pub const MAX_PLAYER_CRITICAL_HIT_ATTEMPT_CALLBACKS: usize = 64;
/// Maximum finite damage multiplier represented in a critical-hit callback.
pub const MAX_PLAYER_CRITICAL_HIT_ATTEMPT_MULTIPLIER: f32 = 1_000.0;
/// Cancellation grant required to clear the critical-hit flag; `events.subscribe` is also required.
pub const PLAYER_CRITICAL_HIT_ATTEMPT_CANCEL_CAPABILITY: &str = "events.cancel";

/// Synchronous logical-server values observed when a player attack is considered
/// for a critical hit.
///
/// The host validates that IDs are canonical and that both multipliers are
/// finite values between 0.0 and 1,000.0 before the callback runs.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerCriticalHitAttempt {
    /// Canonical UUID of the attacking player.
    pub player_id: String,
    /// Resource location of the dimension containing the attack.
    pub dimension_id: String,
    /// Canonical UUID of the attacked entity.
    pub target_id: String,
    /// Registry ID of the attacked entity type.
    pub target_type_id: String,
    /// Whether vanilla originally marked this attack as a critical hit.
    pub vanilla_critical: bool,
    /// Critical damage multiplier originally assigned by vanilla.
    pub vanilla_multiplier: f32,
    /// Damage multiplier currently assigned before guest callbacks run.
    pub damage_multiplier: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_bytes, to_bytes, ErrorCode};

    fn sample() -> PlayerCriticalHitAttempt {
        PlayerCriticalHitAttempt {
            player_id: "00000000-0000-0000-0000-000000000001".into(),
            dimension_id: "minecraft:overworld".into(),
            target_id: "00000000-0000-0000-0000-000000000002".into(),
            target_type_id: "minecraft:cow".into(),
            vanilla_critical: true,
            vanilla_multiplier: 1.5,
            damage_multiplier: 1.25,
        }
    }

    fn expected_wire(event: &PlayerCriticalHitAttempt) -> Vec<u8> {
        let mut output = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut output, &event.player_id);
        append_test_string(&mut output, &event.dimension_id);
        append_test_string(&mut output, &event.target_id);
        append_test_string(&mut output, &event.target_type_id);
        output.push(u8::from(event.vanilla_critical));
        output.extend_from_slice(&event.vanilla_multiplier.to_bits().to_le_bytes());
        output.extend_from_slice(&event.damage_multiplier.to_bits().to_le_bytes());
        output
    }

    #[test]
    fn critical_hit_attempt_matches_the_java_wire_layout_and_round_trips() {
        let event = sample();
        let expected = expected_wire(&event);
        assert_eq!(to_bytes(&event).unwrap(), expected);
        assert_eq!(
            from_bytes::<PlayerCriticalHitAttempt>(&expected).unwrap(),
            event
        );

        let mut non_critical = sample();
        non_critical.vanilla_critical = false;
        let expected = expected_wire(&non_critical);
        assert_eq!(to_bytes(&non_critical).unwrap(), expected);
        assert_eq!(
            from_bytes::<PlayerCriticalHitAttempt>(&expected).unwrap(),
            non_critical
        );
    }

    #[test]
    fn critical_hit_attempt_rejects_foreign_schema_and_boolean_bytes() {
        let mut wrong_version = to_bytes(&sample()).unwrap();
        wrong_version[..4].copy_from_slice(&2_i32.to_le_bytes());
        assert_eq!(
            from_bytes::<PlayerCriticalHitAttempt>(&wrong_version)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );

        let event = sample();
        let mut wrong_boolean = to_bytes(&event).unwrap();
        let flag_offset = 4
            + (4 + event.player_id.len())
            + (4 + event.dimension_id.len())
            + (4 + event.target_id.len())
            + (4 + event.target_type_id.len());
        wrong_boolean[flag_offset] = 2;
        assert_eq!(
            from_bytes::<PlayerCriticalHitAttempt>(&wrong_boolean)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidBoolean
        );
    }

    fn append_test_string(output: &mut Vec<u8>, value: &str) {
        output.extend_from_slice(&(value.len() as i32).to_le_bytes());
        output.extend_from_slice(value.as_bytes());
    }
}
