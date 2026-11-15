//! Detached payload for cancellable logical-server note block note changes.
//!
//! Register a `block.note_block_change_attempt` callback with `events.subscribe`
//! and `events.cancel`. Returning `Ok(true)` cancels the note change and keeps
//! the note block silent. The callback is synchronous because NeoForge checks
//! the cancellation while it is still dispatching the change event.

use crate::{StableDecode, StableEncode};

/// Event name subscribed with `events::subscribe`.
pub const BLOCK_NOTE_BLOCK_CHANGE_ATTEMPT_EVENT_NAME: &str = "block.note_block_change_attempt";
/// Cancellation grant required to veto note block changes; `events.subscribe` is also required.
pub const BLOCK_NOTE_BLOCK_CHANGE_ATTEMPT_CANCEL_CAPABILITY: &str = "events.cancel";
/// Maximum UTF-8 byte length of a dimension ID.
pub const MAX_BLOCK_NOTE_BLOCK_CHANGE_ATTEMPT_ID_BYTES: usize = 256;
/// Maximum note ordinal carried by a note block change attempt.
pub const MAX_BLOCK_NOTE_BLOCK_CHANGE_ATTEMPT_NOTE: i32 = 24;
/// Maximum octave ordinal carried by a note block change attempt.
pub const MAX_BLOCK_NOTE_BLOCK_CHANGE_ATTEMPT_OCTAVE: i32 = 2;
/// Maximum absolute block x/z coordinate accepted in a note block change attempt.
pub const MAX_BLOCK_NOTE_BLOCK_CHANGE_ATTEMPT_COORDINATE: i32 = 30_000_000;
/// Maximum serialized payload size for one note block change attempt callback.
pub const MAX_BLOCK_NOTE_BLOCK_CHANGE_ATTEMPT_PAYLOAD_BYTES: usize = 1_024;
/// Maximum callbacks invoked for one note block change attempt.
pub const MAX_BLOCK_NOTE_BLOCK_CHANGE_ATTEMPT_CALLBACKS: usize = 64;

/// Synchronous cancellable snapshot captured when a note block note is changed.
///
/// NeoForge fires `block.note_block_change_attempt` before vanilla applies the
/// new note. Return `Ok(true)` to cancel the change, which also stops the note
/// block from playing. The detached snapshot contains the dimension, block
/// position, the previous note ordinal with its octave ordinal, and the new
/// note ordinal.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct BlockNoteBlockChangeAttempt {
    /// Resource location of the dimension containing the note block.
    pub dimension_id: String,
    /// Note block x coordinate.
    pub x: i32,
    /// Note block y coordinate.
    pub y: i32,
    /// Note block z coordinate.
    pub z: i32,
    /// Previous note ordinal, bounded by the host to `0..=24`.
    pub old_note: i32,
    /// Previous octave ordinal, bounded by the host to `0..=2`.
    pub old_octave: i32,
    /// New note ordinal, bounded by the host to `0..=24`.
    pub new_note: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_bytes, to_bytes, ErrorCode};

    fn sample() -> BlockNoteBlockChangeAttempt {
        BlockNoteBlockChangeAttempt {
            dimension_id: "minecraft:overworld".into(),
            x: 1,
            y: 64,
            z: -3,
            old_note: 5,
            old_octave: 0,
            new_note: 7,
        }
    }

    fn expected_wire(event: &BlockNoteBlockChangeAttempt) -> Vec<u8> {
        let mut output = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut output, &event.dimension_id);
        output.extend_from_slice(&event.x.to_le_bytes());
        output.extend_from_slice(&event.y.to_le_bytes());
        output.extend_from_slice(&event.z.to_le_bytes());
        output.extend_from_slice(&event.old_note.to_le_bytes());
        output.extend_from_slice(&event.old_octave.to_le_bytes());
        output.extend_from_slice(&event.new_note.to_le_bytes());
        output
    }

    #[test]
    fn note_block_change_attempt_matches_the_java_wire_layout_and_round_trips() {
        let event = sample();
        let expected = expected_wire(&event);
        assert_eq!(to_bytes(&event).unwrap(), expected);
        assert_eq!(
            from_bytes::<BlockNoteBlockChangeAttempt>(&expected).unwrap(),
            event
        );
    }

    #[test]
    fn note_block_change_attempt_round_trips_negative_coordinates_and_note_bounds() {
        let mut event = sample();
        event.x = -30_000_000;
        event.y = -64;
        event.z = 30_000_000;
        event.old_note = 0;
        event.old_octave = 2;
        event.new_note = 24;
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(
            from_bytes::<BlockNoteBlockChangeAttempt>(&encoded).unwrap(),
            event
        );
    }

    #[test]
    fn note_block_change_attempt_rejects_other_schema_versions() {
        let mut encoded = to_bytes(&sample()).unwrap();
        encoded[..4].copy_from_slice(&2_i32.to_le_bytes());
        assert_eq!(
            from_bytes::<BlockNoteBlockChangeAttempt>(&encoded)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );
    }

    #[test]
    fn note_block_change_attempt_constants_match_the_host_contract() {
        assert_eq!(
            BLOCK_NOTE_BLOCK_CHANGE_ATTEMPT_EVENT_NAME,
            "block.note_block_change_attempt"
        );
        assert_eq!(
            BLOCK_NOTE_BLOCK_CHANGE_ATTEMPT_CANCEL_CAPABILITY,
            "events.cancel"
        );
        assert_eq!(MAX_BLOCK_NOTE_BLOCK_CHANGE_ATTEMPT_ID_BYTES, 256);
        assert_eq!(MAX_BLOCK_NOTE_BLOCK_CHANGE_ATTEMPT_NOTE, 24);
        assert_eq!(MAX_BLOCK_NOTE_BLOCK_CHANGE_ATTEMPT_OCTAVE, 2);
        assert_eq!(MAX_BLOCK_NOTE_BLOCK_CHANGE_ATTEMPT_COORDINATE, 30_000_000);
        assert_eq!(MAX_BLOCK_NOTE_BLOCK_CHANGE_ATTEMPT_PAYLOAD_BYTES, 1_024);
        assert_eq!(MAX_BLOCK_NOTE_BLOCK_CHANGE_ATTEMPT_CALLBACKS, 64);
    }

    fn append_test_string(output: &mut Vec<u8>, value: &str) {
        output.extend_from_slice(&(value.len() as i32).to_le_bytes());
        output.extend_from_slice(value.as_bytes());
    }
}
