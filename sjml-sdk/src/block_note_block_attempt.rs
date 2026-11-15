//! Detached payload for cancellable logical-server note block plays.
//!
//! Register a `block.note_block_attempt` callback with `events.subscribe` and
//! `events.cancel`. Returning `Ok(true)` cancels the note sound and particle.
//! The callback is synchronous because NeoForge checks the cancellation while
//! it is still dispatching the play event.

use crate::{StableDecode, StableEncode};

/// Event name subscribed with `events::subscribe`.
pub const BLOCK_NOTE_BLOCK_ATTEMPT_EVENT_NAME: &str = "block.note_block_attempt";
/// Cancellation grant required to veto note block plays; `events.subscribe` is also required.
pub const BLOCK_NOTE_BLOCK_ATTEMPT_CANCEL_CAPABILITY: &str = "events.cancel";
/// Maximum UTF-8 byte length of a dimension ID.
pub const MAX_BLOCK_NOTE_BLOCK_ATTEMPT_ID_BYTES: usize = 256;
/// Maximum UTF-8 byte length of a lowercase note block instrument name.
pub const MAX_BLOCK_NOTE_BLOCK_ATTEMPT_INSTRUMENT_BYTES: usize = 32;
/// Maximum vanilla note ID carried by a note block attempt.
pub const MAX_BLOCK_NOTE_BLOCK_ATTEMPT_NOTE: i32 = 24;
/// Maximum octave ordinal carried by a note block attempt.
pub const MAX_BLOCK_NOTE_BLOCK_ATTEMPT_OCTAVE: i32 = 2;
/// Maximum absolute block x/z coordinate accepted in a note block attempt.
pub const MAX_BLOCK_NOTE_BLOCK_ATTEMPT_COORDINATE: i32 = 30_000_000;
/// Maximum serialized payload size for one note block attempt callback.
pub const MAX_BLOCK_NOTE_BLOCK_ATTEMPT_PAYLOAD_BYTES: usize = 1_024;
/// Maximum callbacks invoked for one note block play attempt.
pub const MAX_BLOCK_NOTE_BLOCK_ATTEMPT_CALLBACKS: usize = 64;

/// Synchronous cancellable snapshot captured when a note block is about to play.
///
/// NeoForge fires `block.note_block_attempt` before vanilla emits the note
/// sound and particle. Return `Ok(true)` to cancel both. The detached snapshot
/// contains the dimension, block position, lowercase instrument enum name, and
/// the vanilla note ID with its octave ordinal.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct BlockNoteBlockAttempt {
    /// Resource location of the dimension containing the note block.
    pub dimension_id: String,
    /// Note block x coordinate.
    pub x: i32,
    /// Note block y coordinate.
    pub y: i32,
    /// Note block z coordinate.
    pub z: i32,
    /// Lowercase instrument enum name, such as `bell` or `harp`.
    pub instrument: String,
    /// Vanilla note ID, bounded by the host to `0..=24`.
    pub note: i32,
    /// Octave ordinal, bounded by the host to `0..=2`.
    pub octave: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_bytes, to_bytes, ErrorCode};

    fn sample() -> BlockNoteBlockAttempt {
        BlockNoteBlockAttempt {
            dimension_id: "minecraft:overworld".into(),
            x: 1,
            y: 64,
            z: -3,
            instrument: "bell".into(),
            note: 12,
            octave: 1,
        }
    }

    fn expected_wire(event: &BlockNoteBlockAttempt) -> Vec<u8> {
        let mut output = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut output, &event.dimension_id);
        output.extend_from_slice(&event.x.to_le_bytes());
        output.extend_from_slice(&event.y.to_le_bytes());
        output.extend_from_slice(&event.z.to_le_bytes());
        append_test_string(&mut output, &event.instrument);
        output.extend_from_slice(&event.note.to_le_bytes());
        output.extend_from_slice(&event.octave.to_le_bytes());
        output
    }

    #[test]
    fn note_block_attempt_matches_the_java_wire_layout_and_round_trips() {
        let event = sample();
        let expected = expected_wire(&event);
        assert_eq!(to_bytes(&event).unwrap(), expected);
        assert_eq!(
            from_bytes::<BlockNoteBlockAttempt>(&expected).unwrap(),
            event
        );
    }

    #[test]
    fn note_block_attempt_round_trips_negative_coordinates_and_note_bounds() {
        let mut event = sample();
        event.x = -30_000_000;
        event.y = -64;
        event.z = 30_000_000;
        event.instrument = "custom_head".into();
        event.note = 0;
        event.octave = 0;
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(
            from_bytes::<BlockNoteBlockAttempt>(&encoded).unwrap(),
            event
        );
    }

    #[test]
    fn note_block_attempt_rejects_other_schema_versions() {
        let mut encoded = to_bytes(&sample()).unwrap();
        encoded[..4].copy_from_slice(&2_i32.to_le_bytes());
        assert_eq!(
            from_bytes::<BlockNoteBlockAttempt>(&encoded)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );
    }

    #[test]
    fn note_block_attempt_constants_match_the_host_contract() {
        assert_eq!(
            BLOCK_NOTE_BLOCK_ATTEMPT_EVENT_NAME,
            "block.note_block_attempt"
        );
        assert_eq!(BLOCK_NOTE_BLOCK_ATTEMPT_CANCEL_CAPABILITY, "events.cancel");
        assert_eq!(MAX_BLOCK_NOTE_BLOCK_ATTEMPT_ID_BYTES, 256);
        assert_eq!(MAX_BLOCK_NOTE_BLOCK_ATTEMPT_INSTRUMENT_BYTES, 32);
        assert_eq!(MAX_BLOCK_NOTE_BLOCK_ATTEMPT_NOTE, 24);
        assert_eq!(MAX_BLOCK_NOTE_BLOCK_ATTEMPT_OCTAVE, 2);
        assert_eq!(MAX_BLOCK_NOTE_BLOCK_ATTEMPT_COORDINATE, 30_000_000);
        assert_eq!(MAX_BLOCK_NOTE_BLOCK_ATTEMPT_PAYLOAD_BYTES, 1_024);
        assert_eq!(MAX_BLOCK_NOTE_BLOCK_ATTEMPT_CALLBACKS, 64);
    }

    fn append_test_string(output: &mut Vec<u8>, value: &str) {
        output.extend_from_slice(&(value.len() as i32).to_le_bytes());
        output.extend_from_slice(value.as_bytes());
    }
}
