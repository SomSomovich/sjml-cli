use sjml::{
    command, player, player_statistics, ErrorCode, PlayerCustomStat, PlayerStatistic, SjmlError,
    SjmlResult, StableDecode, StatisticFamily,
};

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

#[command(name = "player.statistics.set")]
fn set_and_restore_deaths(input: PlayerId) -> SjmlResult<String> {
    let handle = player_handle(&input.uuid)?;
    let before = player::custom_stat(handle, PlayerCustomStat::Deaths)?;
    let desired = if before == 0 { 7 } else { 0 };
    let applied = player::set_custom_stat(handle, PlayerCustomStat::Deaths, desired)?;
    if applied != desired || player::custom_stat(handle, PlayerCustomStat::Deaths)? != desired {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "vanilla custom-stat setter did not apply the requested counter value",
        ));
    }
    let restored = player::set_custom_stat(handle, PlayerCustomStat::Deaths, before)?;
    if restored != before || player::custom_stat(handle, PlayerCustomStat::Deaths)? != before {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "vanilla custom-stat setter did not restore the prior counter value",
        ));
    }
    Ok("custom statistic value set and restored".to_owned())
}

fn increment_and_verify(
    handle: sjml::PlayerHandle,
    family: StatisticFamily,
    registry_id: &str,
) -> SjmlResult<()> {
    let statistic = PlayerStatistic::new(family, registry_id)?;
    let before = player_statistics::read(handle, &statistic)?;
    if before >= i32::MAX as u32 {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "registry-backed statistic is already at Minecraft's signed integer limit",
        ));
    }
    player_statistics::increment(handle, &statistic, 1)?;
    if player_statistics::read(handle, &statistic)? != before + 1 {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "registry-backed statistic increment did not round-trip",
        ));
    }
    Ok(())
}

#[command(name = "player.statistics.registry.blocks")]
fn check_block_statistics(input: PlayerId) -> SjmlResult<String> {
    let handle = player_handle(&input.uuid)?;
    increment_and_verify(handle, StatisticFamily::BlockMined, "minecraft:stone")?;
    Ok("block-mined statistic incremented".to_owned())
}

#[command(name = "player.statistics.registry.items")]
fn check_item_statistics(input: PlayerId) -> SjmlResult<String> {
    let handle = player_handle(&input.uuid)?;
    for (family, registry_id) in [
        (StatisticFamily::ItemUsed, "minecraft:diamond"),
        (StatisticFamily::ItemBroken, "minecraft:diamond"),
        (StatisticFamily::ItemCrafted, "minecraft:diamond"),
    ] {
        increment_and_verify(handle, family, registry_id)?;
    }
    Ok("item statistics incremented".to_owned())
}

#[command(name = "player.statistics.registry.entities")]
fn check_entity_statistics(input: PlayerId) -> SjmlResult<String> {
    let handle = player_handle(&input.uuid)?;
    for (family, registry_id) in [
        (StatisticFamily::EntityKilled, "minecraft:zombie"),
        (StatisticFamily::EntityKilledBy, "minecraft:zombie"),
    ] {
        increment_and_verify(handle, family, registry_id)?;
    }
    Ok("entity statistics incremented".to_owned())
}

fn set_and_restore(
    handle: sjml::PlayerHandle,
    family: StatisticFamily,
    registry_id: &str,
) -> SjmlResult<()> {
    let statistic = PlayerStatistic::new(family, registry_id)?;
    let before = player_statistics::read(handle, &statistic)?;
    let desired = if before == 0 { 11 } else { 0 };
    let applied = player_statistics::set_value(handle, &statistic, desired)?;
    if applied != desired || player_statistics::read(handle, &statistic)? != desired {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "registry-backed statistic setter did not apply the requested value",
        ));
    }
    let restored = player_statistics::set_value(handle, &statistic, before)?;
    if restored != before || player_statistics::read(handle, &statistic)? != before {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "registry-backed statistic setter did not restore the prior value",
        ));
    }
    Ok(())
}

#[command(name = "player.statistics.registry.set")]
fn set_and_restore_registry_statistics(input: PlayerId) -> SjmlResult<String> {
    let handle = player_handle(&input.uuid)?;
    for (family, id) in [
        (StatisticFamily::BlockMined, "minecraft:stone"),
        (StatisticFamily::ItemUsed, "minecraft:diamond"),
        (StatisticFamily::ItemBroken, "minecraft:diamond"),
        (StatisticFamily::ItemCrafted, "minecraft:diamond"),
        (StatisticFamily::EntityKilled, "minecraft:zombie"),
        (StatisticFamily::EntityKilledBy, "minecraft:zombie"),
    ] {
        set_and_restore(handle, family, id)?;
    }
    Ok("registry-backed statistic values set and restored".to_owned())
}
