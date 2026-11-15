//! Detached observation of chunks transmitted to a server player's client.
//!
//! Subscribe to `world.chunk_watch_sent` with `events::subscribe`. The callback is
//! observational: it returns no veto, runs on the owning logical-server thread, and never
//! loads, pins, or retains the source chunk. It fires only after NeoForge has sent the chunk,
//! so follow-up data may safely be addressed to the now-known chunk.

use sjml::{StableDecode, StableEncode};

/// Event name emitted when a chunk was transmitted to a server player's client.
pub const WORLD_CHUNK_WATCH_SENT_EVENT_NAME: &str = "world.chunk_watch_sent";
/// Guest export the host invokes for one `world.chunk_watch_sent` callback.
pub const WORLD_CHUNK_WATCH_SENT_EXPORT_NAME: &str = "sjml_v1_event_world.chunk_watch_sent";
/// Maximum UTF-8 byte length of a player UUID or dimension resource location.
pub const MAX_WORLD_CHUNK_WATCH_SENT_STRING_BYTES: usize = 256;
/// Maximum absolute chunk X or Z coordinate representable by these callbacks.
pub const MAX_WORLD_CHUNK_WATCH_SENT_COORDINATE: i32 = 4_000_000;
/// Maximum callbacks the host invokes for one chunk transmission event.
pub const MAX_WORLD_CHUNK_WATCH_SENT_CALLBACKS: usize = 64;
/// Maximum encoded payload size for one detached chunk transmission callback.
pub const MAX_WORLD_CHUNK_WATCH_SENT_PAYLOAD_BYTES: usize = 512;

/// Observation-only snapshot for `world.chunk_watch_sent`.
///
/// The player has received the chunk and NeoForge has finished transmitting it. The payload
/// contains no `ServerPlayer`, level, or chunk handle and is safe to keep after the source
/// chunk leaves the player's watch set.
#[derive(Debug, Clone, PartialEq, Eq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct WorldChunkWatchSent {
    /// Canonical resource location of the level that owns the transmitted chunk.
    pub dimension_id: String,
    /// Canonical lowercase UUID of the server player that received the chunk.
    pub player_id: String,
    /// Chunk X coordinate.
    pub chunk_x: i32,
    /// Chunk Z coordinate.
    pub chunk_z: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sjml::{from_bytes, to_bytes};

    fn event() -> WorldChunkWatchSent {
        WorldChunkWatchSent {
            dimension_id: "minecraft:the_nether".into(),
            player_id: "00000000-0000-0000-0000-000000000001".into(),
            chunk_x: -12,
            chunk_z: 345,
        }
    }

    #[test]
    fn event_matches_the_java_wire_layout() {
        let event = event();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_string(&mut expected, &event.dimension_id);
        append_string(&mut expected, &event.player_id);
        expected.extend_from_slice(&event.chunk_x.to_le_bytes());
        expected.extend_from_slice(&event.chunk_z.to_le_bytes());

        assert_eq!(to_bytes(&event).unwrap(), expected);
        assert_eq!(from_bytes::<WorldChunkWatchSent>(&expected).unwrap(), event);
    }

    #[test]
    fn event_round_trips_the_extreme_bounded_coordinates() {
        let mut event = event();
        event.chunk_x = -MAX_WORLD_CHUNK_WATCH_SENT_COORDINATE;
        event.chunk_z = MAX_WORLD_CHUNK_WATCH_SENT_COORDINATE;

        assert_eq!(
            from_bytes::<WorldChunkWatchSent>(&to_bytes(&event).unwrap()).unwrap(),
            event
        );
    }

    #[test]
    fn constants_match_the_documented_contract() {
        assert_eq!(WORLD_CHUNK_WATCH_SENT_EVENT_NAME, "world.chunk_watch_sent");
        assert_eq!(
            WORLD_CHUNK_WATCH_SENT_EXPORT_NAME,
            "sjml_v1_event_world.chunk_watch_sent"
        );
        assert_eq!(MAX_WORLD_CHUNK_WATCH_SENT_STRING_BYTES, 256);
        assert_eq!(MAX_WORLD_CHUNK_WATCH_SENT_COORDINATE, 4_000_000);
        assert_eq!(MAX_WORLD_CHUNK_WATCH_SENT_CALLBACKS, 64);
        assert_eq!(MAX_WORLD_CHUNK_WATCH_SENT_PAYLOAD_BYTES, 512);
    }

    fn append_string(output: &mut Vec<u8>, value: &str) {
        output.extend_from_slice(&(value.len() as u32).to_le_bytes());
        output.extend_from_slice(value.as_bytes());
    }
}
