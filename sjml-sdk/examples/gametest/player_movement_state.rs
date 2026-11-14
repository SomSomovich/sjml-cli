use sjml::{player, ErrorCode, PlayerHandle, SjmlError, SjmlResult};

/// Exercises the detached movement-state read from an existing GameTest command.
pub(crate) fn verify_movement_state_read(handle: PlayerHandle) -> SjmlResult<()> {
    let first = player::movement_state(handle)?;
    let second = player::movement_state(handle)?;
    if first != second {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "read-only player movement state changed between synchronous snapshots",
        ));
    }
    Ok(())
}
