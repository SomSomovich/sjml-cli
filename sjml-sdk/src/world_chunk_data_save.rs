//! Detached observation of a logical-server chunk data save.
//!
//! Subscribe to `world.chunk_data_save` with `events::subscribe`. The callback is
//! observational: it returns no veto, runs on the owning logical-server thread, and never
//! receives, loads, or retains the source chunk or its NBT.

use sjml::{StableDecode, StableEncode};

/// Event name emitted when a chunk's serialized data is about to be written.
pub const WORLD_CHUNK_DATA_SAVE_EVENT_NAME: &str = "world.chunk_data_save";
/// Guest export the host invokes for one `world.chunk_data_save` callback.
pub const WORLD_CHUNK_DATA_SAVE_EXPORT_NAME: &str = "sjml_v1_event_world.chunk_data_save";
/// Maximum UTF-8 byte length of a dimension resource location.
pub const MAX_WORLD_CHUNK_DATA_SAVE_STRING_BYTES: usize = 256;
/// Maximum absolute chunk X or Z coordinate representable by this callback.
pub const MAX_WORLD_CHUNK_DATA_SAVE_COORDINATE: i32 = 4_000_000;
/// Maximum serialized top-level NBT entry count representable by this callback.
pub const MAX_WORLD_CHUNK_DATA_SAVE_DATA_ENTRY_COUNT: i32 = 100_000_000;
/// Maximum callbacks the host invokes for one chunk data save event.
pub const MAX_WORLD_CHUNK_DATA_SAVE_CALLBACKS: usize = 64;
/// Maximum encoded payload size for one detached chunk data save callback.
pub const MAX_WORLD_CHUNK_DATA_SAVE_PAYLOAD_BYTES: usize = 512;

/// Observation-only snapshot for `world.chunk_data_save`.
///
/// NeoForge reports the event immediately before the chunk's NBT is handed to storage. The
/// payload contains only detached metadata: no compound tag, level, or chunk handle is exposed
/// and nothing here can alter, delay, or cancel the write.
#[derive(Debug, Clone, PartialEq, Eq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct WorldChunkDataSave {
    /// Canonical resource location of the level that owns the saved chunk.
    pub dimension_id: String,
    /// Chunk X coordinate.
    pub chunk_x: i32,
    /// Chunk Z coordinate.
    pub chunk_z: i32,
    /// Number of top-level entries in the serialized chunk NBT compound tag.
    pub data_entry_count: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sjml::{from_bytes, to_bytes};

    fn event() -> WorldChunkDataSave {
        WorldChunkDataSave {
            dimension_id: "minecraft:the_nether".into(),
            chunk_x: -12,
            chunk_z: 345,
            data_entry_count: 42,
        }
    }

    #[test]
    fn event_matches_the_java_wire_layout() {
        let event = event();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_string(&mut expected, &event.dimension_id);
        expected.extend_from_slice(&event.chunk_x.to_le_bytes());
        expected.extend_from_slice(&event.chunk_z.to_le_bytes());
        expected.extend_from_slice(&event.data_entry_count.to_le_bytes());

        assert_eq!(to_bytes(&event).unwrap(), expected);
        assert_eq!(from_bytes::<WorldChunkDataSave>(&expected).unwrap(), event);
    }

    #[test]
    fn event_round_trips_the_extreme_bounded_values() {
        let mut event = event();
        event.chunk_x = -MAX_WORLD_CHUNK_DATA_SAVE_COORDINATE;
        event.chunk_z = MAX_WORLD_CHUNK_DATA_SAVE_COORDINATE;
        event.data_entry_count = MAX_WORLD_CHUNK_DATA_SAVE_DATA_ENTRY_COUNT;

        assert_eq!(
            from_bytes::<WorldChunkDataSave>(&to_bytes(&event).unwrap()).unwrap(),
            event
        );

        event.data_entry_count = 0;
        assert_eq!(
            from_bytes::<WorldChunkDataSave>(&to_bytes(&event).unwrap()).unwrap(),
            event
        );
    }

    #[test]
    fn constants_match_the_documented_contract() {
        assert_eq!(WORLD_CHUNK_DATA_SAVE_EVENT_NAME, "world.chunk_data_save");
        assert_eq!(
            WORLD_CHUNK_DATA_SAVE_EXPORT_NAME,
            "sjml_v1_event_world.chunk_data_save"
        );
        assert_eq!(MAX_WORLD_CHUNK_DATA_SAVE_STRING_BYTES, 256);
        assert_eq!(MAX_WORLD_CHUNK_DATA_SAVE_COORDINATE, 4_000_000);
        assert_eq!(MAX_WORLD_CHUNK_DATA_SAVE_DATA_ENTRY_COUNT, 100_000_000);
        assert_eq!(MAX_WORLD_CHUNK_DATA_SAVE_CALLBACKS, 64);
        assert_eq!(MAX_WORLD_CHUNK_DATA_SAVE_PAYLOAD_BYTES, 512);
    }

    fn append_string(output: &mut Vec<u8>, value: &str) {
        output.extend_from_slice(&(value.len() as u32).to_le_bytes());
        output.extend_from_slice(value.as_bytes());
    }
}
