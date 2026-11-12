use sjml::{command, player, ErrorCode, PlayerCustomStat, SjmlError, SjmlResult, StableDecode};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct PlayerId {
    uuid: String,
}

fn player_handle(uuid: &str) -> SjmlResult<sjml::PlayerHandle> {
    for handle in player::online()? {
        if player::snapshot(handle)?.uuid == uuid {
            return Ok(handle);
        }
    }
    Err(SjmlError::new(
        ErrorCode::InvalidArgument,
        "GameTest player was not present in the live player list",
    ))
}

#[command(name = "player.statistics.increment")]
fn increment_deaths(input: PlayerId) -> SjmlResult<String> {
    let handle = player_handle(&input.uuid)?;
    let before = player::custom_stat(handle, PlayerCustomStat::Deaths)?;
    if before >= i32::MAX as u32 {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "death statistic is already at Minecraft's signed integer limit",
        ));
    }
    player::increment_custom_stat(handle, PlayerCustomStat::Deaths, 1)?;
    let after = player::custom_stat(handle, PlayerCustomStat::Deaths)?;
    if after != before + 1 {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "vanilla custom-stat increment did not change the live counter by one",
        ));
    }
    Ok(format!(
        "death counter incremented from {before} to {after}"
    ))
}
