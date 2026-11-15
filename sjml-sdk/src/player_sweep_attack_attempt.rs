//! Synchronous sweep-attack veto values for `player.sweep_attack_attempt`.
//!
//! NeoForge fires the event on both logical sides after a player attacks a target and the
//! critical-hit decision is complete. Return `Ok(true)` to cancel the sweep: the host cancels
//! the event and disables sweeping, so the attack deals no sweep damage. Register the callback
//! with `events.subscribe` and grant `events.cancel`; asynchronous handlers are rejected because
//! the sweep result must be known before the event returns.

use crate::{StableDecode, StableEncode};

/// Event name for a synchronous sweep-attack veto.
pub const PLAYER_SWEEP_ATTACK_ATTEMPT_EVENT_NAME: &str = "player.sweep_attack_attempt";
/// Cancellation grant required to veto sweep attacks; `events.subscribe` is also required.
pub const PLAYER_SWEEP_ATTACK_ATTEMPT_CANCEL_CAPABILITY: &str = "events.cancel";
/// Maximum UTF-8 byte length of one detached UUID, resource location, or type ID.
pub const MAX_PLAYER_SWEEP_ATTACK_ATTEMPT_STRING_BYTES: usize = 256;
/// Maximum callbacks invoked for one sweep-attack event.
pub const MAX_PLAYER_SWEEP_ATTACK_ATTEMPT_CALLBACKS: usize = 64;
/// Maximum schema-v1 payload size for one sweep-attack callback.
pub const MAX_PLAYER_SWEEP_ATTACK_ATTEMPT_PAYLOAD_BYTES: usize = 2_048;

/// Detached logical-server values observed when a player's attack would sweep.
///
/// All fields are detached strings and flags; no `Player`, target entity, level, or host handle
/// crosses the guest ABI.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerSweepAttackAttempt {
    /// Canonical UUID of the attacking player.
    pub player_id: String,
    /// Resource location of the dimension containing the player and target.
    pub dimension_id: String,
    /// Canonical UUID of the attacked entity.
    pub target_id: String,
    /// Registry ID of the attacked entity type, such as `minecraft:cow`.
    pub target_type_id: String,
    /// Whether vanilla's own sweep rules would trigger a sweep for this attack.
    pub vanilla_sweep: bool,
}

#[cfg(test)]
mod tests {
    use super::{
        PlayerSweepAttackAttempt, MAX_PLAYER_SWEEP_ATTACK_ATTEMPT_CALLBACKS,
        MAX_PLAYER_SWEEP_ATTACK_ATTEMPT_PAYLOAD_BYTES,
        MAX_PLAYER_SWEEP_ATTACK_ATTEMPT_STRING_BYTES,
    };
    use crate::{from_bytes, to_bytes};

    fn sample() -> PlayerSweepAttackAttempt {
        PlayerSweepAttackAttempt {
            player_id: "00000000-0000-0000-0000-000000000001".into(),
            dimension_id: "minecraft:overworld".into(),
            target_id: "00000000-0000-0000-0000-000000000002".into(),
            target_type_id: "minecraft:cow".into(),
            vanilla_sweep: true,
        }
    }

    #[test]
    fn sweep_attack_attempt_matches_the_java_schema_v1_layout() {
        let event = sample();
        let mut expected = 1_u32.to_le_bytes().to_vec();
        append_string(&mut expected, &event.player_id);
        append_string(&mut expected, &event.dimension_id);
        append_string(&mut expected, &event.target_id);
        append_string(&mut expected, &event.target_type_id);
        expected.push(1);
        assert_eq!(to_bytes(&event).unwrap(), expected);
        assert_eq!(
            from_bytes::<PlayerSweepAttackAttempt>(&expected).unwrap(),
            event
        );
    }

    #[test]
    fn sweep_attack_attempt_round_trips_a_disabled_vanilla_sweep() {
        let mut event = sample();
        event.target_type_id = "minecraft:pig".into();
        event.vanilla_sweep = false;
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(
            from_bytes::<PlayerSweepAttackAttempt>(&encoded).unwrap(),
            event
        );
    }

    #[test]
    fn unsupported_schema_versions_are_rejected() {
        let mut encoded = to_bytes(&sample()).unwrap();
        encoded[..4].copy_from_slice(&2_u32.to_le_bytes());
        assert!(from_bytes::<PlayerSweepAttackAttempt>(&encoded).is_err());
    }

    #[test]
    fn abi_bounds_match_the_host_contract() {
        assert_eq!(MAX_PLAYER_SWEEP_ATTACK_ATTEMPT_STRING_BYTES, 256);
        assert_eq!(MAX_PLAYER_SWEEP_ATTACK_ATTEMPT_CALLBACKS, 64);
        assert_eq!(MAX_PLAYER_SWEEP_ATTACK_ATTEMPT_PAYLOAD_BYTES, 2_048);
    }

    fn append_string(output: &mut Vec<u8>, value: &str) {
        output.extend_from_slice(&(value.len() as i32).to_le_bytes());
        output.extend_from_slice(value.as_bytes());
    }
}
