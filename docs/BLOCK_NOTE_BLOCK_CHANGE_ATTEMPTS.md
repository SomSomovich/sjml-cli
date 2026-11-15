# Note block change attempts

Server modules can veto a note block note change before vanilla applies it:

```rust
use sjml::{event, block_note_block_change_attempt::BlockNoteBlockChangeAttempt,
    events::{self, EventPriority}, SjmlResult};

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("block.note_block_change_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "block.note_block_change_attempt")]
fn pin_low_notes(event: BlockNoteBlockChangeAttempt) -> SjmlResult<bool> {
    Ok(event.old_note == 5)
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's cancellable `NoteBlockEvent.Change`. Returning `Ok(true)` cancels the
change entirely: the note block keeps its previous note and does not play the
new one. Returning `Ok(false)` keeps vanilla behavior. The module must request
and receive both `events.subscribe` and `events.cancel`, and `#[event_async]` is
rejected because the veto must apply while the event is still being dispatched.
Events that are already canceled are never dispatched to the guest.

The detached schema-v1 payload is little-endian: `i32` schema version `1`, the
dimension ID, the block x/y/z, the previous note ordinal (`0..=24`), the
previous octave ordinal (`0..=2`), and the new note ordinal (`0..=24`). The x/z
coordinates are bounded to ±30,000,000, each string is limited to 256 UTF-8
bytes, the payload to 1 KiB, and at most 64 subscribed handlers run per attempt.
No `BlockState`, `Level`, or other Java object crosses the ABI.

The dedicated-server GameTest places a note block at the test origin and posts a
real `NoteBlockEvent.Change` with old note `5` and new note `7`, plus an
unmarked control with old note `6` and new note `8`. It verifies that only the
marked change is canceled. Unit tests cover the stable codec bytes,
note/octave/coordinate bounds, the cancellable NeoForge contract, and detached
snapshot values.
