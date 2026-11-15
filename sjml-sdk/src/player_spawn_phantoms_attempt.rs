//! Detached payload and response for cancellable logical-server phantom spawns.
//!
//! Register a `player.spawn_phantoms_attempt` callback with `events.subscribe`
//! and `events.cancel`. Returning `Ok(true)` sets NeoForge's phantom-spawn
//! result to `DENY`, which prevents the spawn attempt for that player. The
//! callback is synchronous because the result is read as soon as the event
//! returns.

use crate::{StableDecode, StableEncode};

/// Event name for phantom spawn attempts on the logical server.
pub const PLAYER_SPAWN_PHANTOMS_ATTEMPT_EVENT_NAME: &str = "player.spawn_phantoms_attempt";
/// Cancellation grant required to veto phantom spawn attempts; `events.subscribe` is also required.
pub const PLAYER_SPAWN_PHANTOMS_ATTEMPT_CANCEL_CAPABILITY: &str = "events.cancel";
/// Maximum UTF-8 byte length for a player, dimension, or UUID identifier.
pub const MAX_PLAYER_SPAWN_PHANTOMS_ATTEMPT_ID_BYTES: usize = 256;
/// Maximum phantom count accepted in one spawn attempt.
pub const MAX_PLAYER_SPAWN_PHANTOMS_ATTEMPT_PHANTOMS: u32 = 64;
/// Maximum world coordinate accepted in one player position.
pub const MAX_PLAYER_SPAWN_PHANTOMS_ATTEMPT_COORDINATE: f64 = 30_000_000.0;
/// Maximum serialized payload size for one phantom-spawn callback.
pub const MAX_PLAYER_SPAWN_PHANTOMS_ATTEMPT_PAYLOAD_BYTES: usize = 1_024;
/// Maximum callbacks invoked for one phantom spawn attempt.
pub const MAX_PLAYER_SPAWN_PHANTOMS_ATTEMPT_CALLBACKS: usize = 64;

/// Synchronous cancellable snapshot captured when phantoms would spawn.
///
/// Returning `Ok(true)` from the handler forbids the spawn attempt: NeoForge's
/// result becomes `DENY`, so no phantoms are placed for that player. Returning
/// `Ok(false)` leaves `DEFAULT`, so vanilla spawn rules apply. The detached
/// snapshot contains the player UUID, the dimension, the pending phantom
/// count, and the player's bounded world position.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerSpawnPhantomsAttempt {
    /// Canonical UUID of the player the phantoms would target.
    pub player_id: String,
    /// Resource location of the dimension containing the player.
    pub dimension_id: String,
    /// Number of phantoms that would spawn, in the range 0..=64.
    pub phantoms_to_spawn: u32,
    /// World-space X coordinate of the player.
    pub x: f64,
    /// World-space Y coordinate of the player.
    pub y: f64,
    /// World-space Z coordinate of the player.
    pub z: f64,
}

#[cfg(test)]
mod tests {
    use super::PlayerSpawnPhantomsAttempt;
    use crate::{from_bytes, to_bytes, ErrorCode};

    fn sample() -> PlayerSpawnPhantomsAttempt {
        PlayerSpawnPhantomsAttempt {
            player_id: "00000000-0000-0000-0000-000000000001".into(),
            dimension_id: "minecraft:overworld".into(),
            phantoms_to_spawn: 3,
            x: 1.5,
            y: 64.0,
            z: -3.25,
        }
    }

    #[test]
    fn spawn_phantoms_attempt_uses_the_stable_schema_v1_wire_layout() {
        let event = sample();
        let encoded = to_bytes(&event).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_u32.to_le_bytes());
        for value in [&event.player_id, &event.dimension_id] {
            expected.extend_from_slice(&(value.len() as u32).to_le_bytes());
            expected.extend_from_slice(value.as_bytes());
        }
        expected.extend_from_slice(&event.phantoms_to_spawn.to_le_bytes());
        for value in [event.x, event.y, event.z] {
            expected.extend_from_slice(&value.to_bits().to_le_bytes());
        }
        assert_eq!(encoded, expected);
        assert_eq!(
            from_bytes::<PlayerSpawnPhantomsAttempt>(&encoded).unwrap(),
            event
        );
    }

    #[test]
    fn spawn_phantoms_attempt_round_trips_negative_positions() {
        let mut event = sample();
        event.x = -29_999_999.5;
        event.y = -64.0;
        event.z = 12_345.678;
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(
            from_bytes::<PlayerSpawnPhantomsAttempt>(&encoded).unwrap(),
            event
        );
    }

    #[test]
    fn spawn_phantoms_attempt_rejects_other_schema_versions() {
        let mut encoded = to_bytes(&sample()).unwrap();
        encoded[..4].copy_from_slice(&2_u32.to_le_bytes());
        assert_eq!(
            from_bytes::<PlayerSpawnPhantomsAttempt>(&encoded)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );
    }

    #[test]
    fn spawn_phantoms_attempt_constants_match_the_host_contract() {
        assert_eq!(
            super::PLAYER_SPAWN_PHANTOMS_ATTEMPT_EVENT_NAME,
            "player.spawn_phantoms_attempt"
        );
        assert_eq!(
            super::PLAYER_SPAWN_PHANTOMS_ATTEMPT_CANCEL_CAPABILITY,
            "events.cancel"
        );
        assert_eq!(super::MAX_PLAYER_SPAWN_PHANTOMS_ATTEMPT_ID_BYTES, 256);
        assert_eq!(super::MAX_PLAYER_SPAWN_PHANTOMS_ATTEMPT_PHANTOMS, 64);
        assert_eq!(
            super::MAX_PLAYER_SPAWN_PHANTOMS_ATTEMPT_PAYLOAD_BYTES,
            1_024
        );
        assert_eq!(super::MAX_PLAYER_SPAWN_PHANTOMS_ATTEMPT_CALLBACKS, 64);
        assert_eq!(
            super::MAX_PLAYER_SPAWN_PHANTOMS_ATTEMPT_COORDINATE,
            30_000_000.0
        );
    }
}
