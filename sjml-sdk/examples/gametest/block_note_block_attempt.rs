use sjml::{
    block_note_block_attempt::BlockNoteBlockAttempt,
    event,
    events::{self, EventPriority},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("block.note_block_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "block.note_block_attempt")]
fn veto_bell_notes(event: BlockNoteBlockAttempt) -> SjmlResult<bool> {
    Ok(event.instrument == "bell")
}
