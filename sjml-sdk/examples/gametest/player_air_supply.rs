use sjml::{player, ErrorCode, PlayerHandle, SjmlError, SjmlResult};

/// Exercises a server player's detached air snapshot and bounded mutation, then restores it.
pub(crate) fn verify_read_set_read_and_restore(handle: PlayerHandle) -> SjmlResult<()> {
    let original = player::air_supply(handle)?;
    if original.current < 0 || original.max == 0 || original.current as u32 > original.max {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "GameTest player air supply must be nonnegative with a positive maximum",
        ));
    }
    let changed = if original.current == 0 {
        1
    } else {
        original.current as u32 - 1
    };

    let change_and_read = (|| {
        player::set_air_supply(handle, changed)?;
        let observed = player::air_supply(handle)?;
        if observed.current != changed as i32 || observed.max != original.max {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "player air-supply read did not match the bounded guest write",
            ));
        }
        Ok(())
    })();
    let restore = player::set_air_supply(handle, original.current as u32);
    restore?;
    change_and_read?;

    if player::air_supply(handle)? != original {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player air supply was not restored after the guest write",
        ));
    }
    Ok(())
}
