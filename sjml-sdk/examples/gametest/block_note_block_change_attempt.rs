use sjml::{
    block_note_block_change_attempt::BlockNoteBlockChangeAttempt,
    event,
    events::{self, EventPriority},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("block.note_block_change_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "block.note_block_change_attempt")]
fn veto_marked_note_changes(event: BlockNoteBlockChangeAttempt) -> SjmlResult<bool> {
    Ok(event.old_note == 5)
}
