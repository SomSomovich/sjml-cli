//! Detached observation of the chunks a server player starts or stops watching.
//!
//! Subscribe to `world.chunk_watch` and `world.chunk_unwatch` with
//! `events::subscribe`. Both callbacks are observational: they return no veto, run on the
//! owning logical-server thread, and never force, load, or retain the source chunk.

use sjml::{StableDecode, StableEncode};

/// Event name emitted when a server player starts watching a chunk.
pub const WORLD_CHUNK_WATCH_EVENT_NAME: &str = "world.chunk_watch";
/// Event name emitted when a server player stops watching a chunk.
pub const WORLD_CHUNK_UNWATCH_EVENT_NAME: &str = "world.chunk_unwatch";
/// Guest export the host invokes for one `world.chunk_watch` callback.
pub const WORLD_CHUNK_WATCH_EXPORT_NAME: &str = "sjml_v1_event_world.chunk_watch";
/// Guest export the host invokes for one `world.chunk_unwatch` callback.
pub const WORLD_CHUNK_UNWATCH_EXPORT_NAME: &str = "sjml_v1_event_world.chunk_unwatch";
/// Maximum UTF-8 byte length of a player UUID or dimension resource location.
pub const MAX_WORLD_CHUNK_WATCH_STRING_BYTES: usize = 256;
/// Maximum absolute chunk X or Z coordinate representable by these callbacks.
pub const MAX_WORLD_CHUNK_WATCH_COORDINATE: i32 = 4_000_000;
/// Maximum callbacks the host invokes for one chunk watch event.
pub const MAX_WORLD_CHUNK_WATCH_CALLBACKS: usize = 64;
/// Maximum encoded payload size for one detached chunk watch callback.
pub const MAX_WORLD_CHUNK_WATCH_PAYLOAD_BYTES: usize = 1_024;

/// Observation-only snapshot for `world.chunk_watch`.
///
/// The player has started watching the chunk and NeoForge has queued it for sending. The
/// payload contains no `ServerPlayer`, level, or chunk handle and is safe to keep after the
/// source chunk leaves the player's watch set.
#[derive(Debug, Clone, PartialEq, Eq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct WorldChunkWatched {
    /// Canonical lowercase UUID of the server player that started watching the chunk.
    pub player_id: String,
    /// Canonical resource location of the level that owns the watched chunk.
    pub dimension_id: String,
    /// Chunk X coordinate.
    pub chunk_x: i32,
    /// Chunk Z coordinate.
    pub chunk_z: i32,
}

/// Observation-only snapshot for `world.chunk_unwatch`.
///
/// The player has stopped watching the chunk. The chunk may never have reached the client if
/// the watch was dropped before it was sent. No live game object crosses the guest ABI.
#[derive(Debug, Clone, PartialEq, Eq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct WorldChunkUnwatched {
    /// Canonical lowercase UUID of the server player that stopped watching the chunk.
    pub player_id: String,
    /// Canonical resource location of the level that owns the unwatched chunk.
    pub dimension_id: String,
    /// Chunk X coordinate.
    pub chunk_x: i32,
    /// Chunk Z coordinate.
    pub chunk_z: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sjml::{from_bytes, to_bytes};

    fn watched() -> WorldChunkWatched {
        WorldChunkWatched {
            player_id: "00000000-0000-0000-0000-000000000001".into(),
            dimension_id: "minecraft:the_nether".into(),
            chunk_x: -12,
            chunk_z: 345,
        }
    }

    fn unwatched() -> WorldChunkUnwatched {
        WorldChunkUnwatched {
            player_id: "00000000-0000-0000-0000-000000000001".into(),
            dimension_id: "minecraft:the_nether".into(),
            chunk_x: -12,
            chunk_z: 345,
        }
    }

    #[test]
    fn watched_event_matches_the_java_wire_layout() {
        let event = watched();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_string(&mut expected, &event.player_id);
        append_string(&mut expected, &event.dimension_id);
        expected.extend_from_slice(&event.chunk_x.to_le_bytes());
        expected.extend_from_slice(&event.chunk_z.to_le_bytes());

        assert_eq!(to_bytes(&event).unwrap(), expected);
        assert_eq!(from_bytes::<WorldChunkWatched>(&expected).unwrap(), event);
    }

    #[test]
    fn unwatched_event_matches_the_java_wire_layout() {
        let event = unwatched();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_string(&mut expected, &event.player_id);
        append_string(&mut expected, &event.dimension_id);
        expected.extend_from_slice(&event.chunk_x.to_le_bytes());
        expected.extend_from_slice(&event.chunk_z.to_le_bytes());

        assert_eq!(to_bytes(&event).unwrap(), expected);
        assert_eq!(from_bytes::<WorldChunkUnwatched>(&expected).unwrap(), event);
    }

    #[test]
    fn events_round_trip_the_extreme_bounded_coordinates() {
        let mut event = watched();
        event.chunk_x = -MAX_WORLD_CHUNK_WATCH_COORDINATE;
        event.chunk_z = MAX_WORLD_CHUNK_WATCH_COORDINATE;

        assert_eq!(
            from_bytes::<WorldChunkWatched>(&to_bytes(&event).unwrap()).unwrap(),
            event
        );
    }

    #[test]
    fn constants_match_the_documented_contract() {
        assert_eq!(WORLD_CHUNK_WATCH_EVENT_NAME, "world.chunk_watch");
        assert_eq!(WORLD_CHUNK_UNWATCH_EVENT_NAME, "world.chunk_unwatch");
        assert_eq!(
            WORLD_CHUNK_WATCH_EXPORT_NAME,
            "sjml_v1_event_world.chunk_watch"
        );
        assert_eq!(
            WORLD_CHUNK_UNWATCH_EXPORT_NAME,
            "sjml_v1_event_world.chunk_unwatch"
        );
        assert_eq!(MAX_WORLD_CHUNK_WATCH_STRING_BYTES, 256);
        assert_eq!(MAX_WORLD_CHUNK_WATCH_COORDINATE, 4_000_000);
        assert_eq!(MAX_WORLD_CHUNK_WATCH_CALLBACKS, 64);
        assert_eq!(MAX_WORLD_CHUNK_WATCH_PAYLOAD_BYTES, 1_024);
    }

    fn append_string(output: &mut Vec<u8>, value: &str) {
        output.extend_from_slice(&(value.len() as u32).to_le_bytes());
        output.extend_from_slice(value.as_bytes());
    }
}
