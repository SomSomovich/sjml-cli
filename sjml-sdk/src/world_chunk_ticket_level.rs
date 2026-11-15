//! Detached observation of logical-server chunk ticket level changes.
//!
//! Subscribe to `world.chunk_ticket_level` with `events::subscribe`. The callback is
//! observational: it returns no veto, runs on the owning logical-server thread, and never
//! loads, pins, or otherwise retains the source chunk. NeoForge may report the same chunk
//! twice in one tick when vanilla stages a ticket level transition.

use sjml::{StableDecode, StableEncode};

/// Event name emitted when a logical-server chunk ticket level changes.
pub const WORLD_CHUNK_TICKET_LEVEL_EVENT_NAME: &str = "world.chunk_ticket_level";
/// Guest export the host invokes for one `world.chunk_ticket_level` callback.
pub const WORLD_CHUNK_TICKET_LEVEL_EXPORT_NAME: &str = "sjml_v1_event_world.chunk_ticket_level";
/// Maximum UTF-8 byte length of a dimension resource location.
pub const MAX_WORLD_CHUNK_TICKET_LEVEL_STRING_BYTES: usize = 256;
/// Maximum absolute chunk X or Z coordinate representable by these callbacks.
pub const MAX_WORLD_CHUNK_TICKET_LEVEL_COORDINATE: i32 = 4_000_000;
/// Smallest supported ticket level.
pub const MIN_WORLD_CHUNK_TICKET_LEVEL: i32 = 0;
/// Largest supported ticket level.
pub const MAX_WORLD_CHUNK_TICKET_LEVEL: i32 = 64;
/// Maximum callbacks the host invokes for one ticket level event.
pub const MAX_WORLD_CHUNK_TICKET_LEVEL_CALLBACKS: usize = 64;
/// Maximum encoded payload size for one detached ticket level callback.
pub const MAX_WORLD_CHUNK_TICKET_LEVEL_PAYLOAD_BYTES: usize = 512;

/// Observation-only snapshot for `world.chunk_ticket_level`.
///
/// The value contains no `ServerLevel`, `ChunkHolder`, chunk, or server handle and is safe to
/// keep after the source event returns. `has_chunk_holder` reports whether NeoForge supplied a
/// live holder for the position; it does not imply the chunk is loaded or ticking.
#[derive(Debug, Clone, PartialEq, Eq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct WorldChunkTicketLevel {
    /// Canonical resource location of the level that owns the chunk.
    pub dimension_id: String,
    /// Chunk X coordinate.
    pub chunk_x: i32,
    /// Chunk Z coordinate.
    pub chunk_z: i32,
    /// Ticket level before the transition.
    pub old_level: i32,
    /// Ticket level after the transition.
    pub new_level: i32,
    /// Whether the source event carried a non-null `ChunkHolder`.
    pub has_chunk_holder: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sjml::{from_bytes, to_bytes};

    fn event() -> WorldChunkTicketLevel {
        WorldChunkTicketLevel {
            dimension_id: "minecraft:the_nether".into(),
            chunk_x: -12,
            chunk_z: 345,
            old_level: 33,
            new_level: 31,
            has_chunk_holder: true,
        }
    }

    #[test]
    fn event_matches_the_java_wire_layout() {
        let event = event();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_string(&mut expected, &event.dimension_id);
        expected.extend_from_slice(&event.chunk_x.to_le_bytes());
        expected.extend_from_slice(&event.chunk_z.to_le_bytes());
        expected.extend_from_slice(&event.old_level.to_le_bytes());
        expected.extend_from_slice(&event.new_level.to_le_bytes());
        expected.push(1);

        assert_eq!(to_bytes(&event).unwrap(), expected);
        assert_eq!(
            from_bytes::<WorldChunkTicketLevel>(&expected).unwrap(),
            event
        );
    }

    #[test]
    fn event_round_trips_boundary_coordinates_and_levels() {
        let mut event = event();
        event.chunk_x = -MAX_WORLD_CHUNK_TICKET_LEVEL_COORDINATE;
        event.chunk_z = MAX_WORLD_CHUNK_TICKET_LEVEL_COORDINATE;
        event.old_level = MIN_WORLD_CHUNK_TICKET_LEVEL;
        event.new_level = MAX_WORLD_CHUNK_TICKET_LEVEL;
        event.has_chunk_holder = false;

        assert_eq!(
            from_bytes::<WorldChunkTicketLevel>(&to_bytes(&event).unwrap()).unwrap(),
            event
        );
    }

    #[test]
    fn constants_match_the_documented_contract() {
        assert_eq!(
            WORLD_CHUNK_TICKET_LEVEL_EVENT_NAME,
            "world.chunk_ticket_level"
        );
        assert_eq!(
            WORLD_CHUNK_TICKET_LEVEL_EXPORT_NAME,
            "sjml_v1_event_world.chunk_ticket_level"
        );
        assert_eq!(MAX_WORLD_CHUNK_TICKET_LEVEL_STRING_BYTES, 256);
        assert_eq!(MAX_WORLD_CHUNK_TICKET_LEVEL_COORDINATE, 4_000_000);
        assert_eq!(MIN_WORLD_CHUNK_TICKET_LEVEL, 0);
        assert_eq!(MAX_WORLD_CHUNK_TICKET_LEVEL, 64);
        assert_eq!(MAX_WORLD_CHUNK_TICKET_LEVEL_CALLBACKS, 64);
        assert_eq!(MAX_WORLD_CHUNK_TICKET_LEVEL_PAYLOAD_BYTES, 512);
    }

    fn append_string(output: &mut Vec<u8>, value: &str) {
        output.extend_from_slice(&(value.len() as u32).to_le_bytes());
        output.extend_from_slice(value.as_bytes());
    }
}
