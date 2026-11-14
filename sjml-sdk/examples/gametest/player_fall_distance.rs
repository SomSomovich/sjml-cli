use sjml::{player, ErrorCode, PlayerHandle, SjmlError, SjmlResult};

/// Resets a fixture's nonzero fall distance and checks the detached post-reset state.
pub(crate) fn verify_reset_fall_distance(handle: PlayerHandle) -> SjmlResult<()> {
    let before = player::movement_state(handle)?;
    if before.fall_distance == 0.0 {
        return Ok(());
    }

    player::reset_fall_distance(handle)?;
    let after = player::movement_state(handle)?;
    if after.fall_distance != 0.0
        || after.on_ground != before.on_ground
        || after.sprinting != before.sprinting
        || after.crouching != before.crouching
        || after.swimming != before.swimming
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "fall-distance reset did not preserve other movement-state fields and set fall distance to zero",
        ));
    }
    Ok(())
}
