# Note block play attempts

Server modules can veto a note block before it plays its sound and particle:

```rust
use sjml::{event, block_note_block_attempt::BlockNoteBlockAttempt,
    events::{self, EventPriority}, SjmlResult};

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("block.note_block_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "block.note_block_attempt")]
fn mute_bells(event: BlockNoteBlockAttempt) -> SjmlResult<bool> {
    Ok(event.instrument == "bell")
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's cancellable `NoteBlockEvent.Play`. Returning `Ok(true)` cancels the
play entirely: neither the note sound nor the note particle appears. Returning
`Ok(false)` keeps vanilla behavior. The module must request and receive both
`events.subscribe` and `events.cancel`, and `#[event_async]` is rejected because
the veto must apply while the event is still being dispatched.

The detached schema-v1 payload is little-endian: `i32` schema version `1`, the
dimension ID, the block x/y/z, the lowercase `NoteBlockInstrument` enum name,
the vanilla note ID (`0..=24`), and the octave ordinal (`0..=2`). The x/z
coordinates are bounded to ±30,000,000, the instrument name matches
`[a-z_]{1,32}`, each string is limited to 256 UTF-8 bytes, the payload to 1 KiB,
and at most 64 subscribed handlers run per attempt. No `BlockState`, `Level`, or
other Java object crosses the ABI.

The dedicated-server GameTest places a note block at the test origin and posts a
real `NoteBlockEvent.Play` for a bell and a harp control. It verifies that only
the marked instrument is canceled. Unit tests cover the stable codec bytes,
note/octave/instrument and coordinate bounds, the cancellable NeoForge contract,
and detached snapshot values.
