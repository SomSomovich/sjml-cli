//! Detached observation of the entities a server player starts or stops tracking.
//!
//! Subscribe to `player.start_tracking` and `player.stop_tracking` with
//! `events::subscribe`. Both callbacks are observational: they return no veto, run on the
//! owning logical-server thread, and never force-load, spawn, or retain the source entity.

use sjml::{StableDecode, StableEncode};

/// Event name emitted when a server player starts tracking an entity.
pub const PLAYER_START_TRACKING_EVENT_NAME: &str = "player.start_tracking";
/// Event name emitted when a server player stops tracking an entity.
pub const PLAYER_STOP_TRACKING_EVENT_NAME: &str = "player.stop_tracking";
/// Guest export the host invokes for one `player.start_tracking` callback.
pub const PLAYER_START_TRACKING_EXPORT_NAME: &str = "sjml_v1_event_player.start_tracking";
/// Guest export the host invokes for one `player.stop_tracking` callback.
pub const PLAYER_STOP_TRACKING_EXPORT_NAME: &str = "sjml_v1_event_player.stop_tracking";
/// Schema version shared by both tracking payloads.
pub const PLAYER_TRACKING_SCHEMA_VERSION: i32 = 1;
/// Maximum UTF-8 byte length of a UUID or resource location in these callbacks.
pub const MAX_PLAYER_TRACKING_STRING_BYTES: usize = 256;
/// Maximum callbacks the host invokes for one tracking event.
pub const MAX_PLAYER_TRACKING_CALLBACKS: usize = 64;
/// Maximum encoded payload size for one detached tracking callback.
pub const MAX_PLAYER_TRACKING_PAYLOAD_BYTES: usize = 768;

/// Observation-only snapshot for `player.start_tracking`.
///
/// The player has started receiving updates about the target entity. The payload contains no
/// `ServerPlayer`, entity, level, or server handle and is safe to keep after the target leaves
/// the player's tracking range.
#[derive(Debug, Clone, PartialEq, Eq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerStartTracking {
    /// Canonical lowercase UUID of the server player that started tracking the entity.
    pub player_id: String,
    /// Canonical resource location of the level that owns the tracking player.
    pub dimension_id: String,
    /// Canonical lowercase UUID of the tracked entity.
    pub target_id: String,
    /// Canonical resource location of the tracked entity's type.
    pub target_type_id: String,
}

/// Observation-only snapshot for `player.stop_tracking`.
///
/// The player no longer receives updates about the target entity. The entity was not
/// necessarily removed from the level; it may have moved out of range or changed dimension.
/// No live game object crosses the guest ABI.
#[derive(Debug, Clone, PartialEq, Eq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerStopTracking {
    /// Canonical lowercase UUID of the server player that stopped tracking the entity.
    pub player_id: String,
    /// Canonical resource location of the level that owns the tracking player.
    pub dimension_id: String,
    /// Canonical lowercase UUID of the untracked entity.
    pub target_id: String,
    /// Canonical resource location of the untracked entity's type.
    pub target_type_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sjml::{from_bytes, to_bytes};

    fn started() -> PlayerStartTracking {
        PlayerStartTracking {
            player_id: "00000000-0000-0000-0000-000000000001".into(),
            dimension_id: "minecraft:the_nether".into(),
            target_id: "123e4567-e89b-12d3-a456-426614174000".into(),
            target_type_id: "minecraft:cow".into(),
        }
    }

    fn stopped() -> PlayerStopTracking {
        PlayerStopTracking {
            player_id: "00000000-0000-0000-0000-000000000001".into(),
            dimension_id: "minecraft:the_nether".into(),
            target_id: "123e4567-e89b-12d3-a456-426614174000".into(),
            target_type_id: "minecraft:cow".into(),
        }
    }

    #[test]
    fn start_tracking_event_matches_the_java_wire_layout() {
        let event = started();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_string(&mut expected, &event.player_id);
        append_string(&mut expected, &event.dimension_id);
        append_string(&mut expected, &event.target_id);
        append_string(&mut expected, &event.target_type_id);

        assert_eq!(to_bytes(&event).unwrap(), expected);
        assert_eq!(from_bytes::<PlayerStartTracking>(&expected).unwrap(), event);
    }

    #[test]
    fn stop_tracking_event_matches_the_java_wire_layout() {
        let event = stopped();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_string(&mut expected, &event.player_id);
        append_string(&mut expected, &event.dimension_id);
        append_string(&mut expected, &event.target_id);
        append_string(&mut expected, &event.target_type_id);

        assert_eq!(to_bytes(&event).unwrap(), expected);
        assert_eq!(from_bytes::<PlayerStopTracking>(&expected).unwrap(), event);
    }

    #[test]
    fn events_round_trip_maximum_bounded_strings() {
        let large = format!("a:{}", "b".repeat(MAX_PLAYER_TRACKING_STRING_BYTES - 2));
        let mut event = started();
        event.dimension_id = large.clone();
        event.target_type_id = large;

        assert_eq!(
            from_bytes::<PlayerStartTracking>(&to_bytes(&event).unwrap()).unwrap(),
            event
        );
    }

    #[test]
    fn constants_match_the_documented_contract() {
        assert_eq!(PLAYER_START_TRACKING_EVENT_NAME, "player.start_tracking");
        assert_eq!(PLAYER_STOP_TRACKING_EVENT_NAME, "player.stop_tracking");
        assert_eq!(
            PLAYER_START_TRACKING_EXPORT_NAME,
            "sjml_v1_event_player.start_tracking"
        );
        assert_eq!(
            PLAYER_STOP_TRACKING_EXPORT_NAME,
            "sjml_v1_event_player.stop_tracking"
        );
        assert_eq!(PLAYER_TRACKING_SCHEMA_VERSION, 1);
        assert_eq!(MAX_PLAYER_TRACKING_STRING_BYTES, 256);
        assert_eq!(MAX_PLAYER_TRACKING_CALLBACKS, 64);
        assert_eq!(MAX_PLAYER_TRACKING_PAYLOAD_BYTES, 768);
    }

    fn append_string(output: &mut Vec<u8>, value: &str) {
        output.extend_from_slice(&(value.len() as u32).to_le_bytes());
        output.extend_from_slice(value.as_bytes());
    }
}
