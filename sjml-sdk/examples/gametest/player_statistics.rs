use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering};

use sjml::{
    command, event,
    events::{self, EventPriority},
    player, player_statistics, ErrorCode, PlayerCustomStat, PlayerStatistic,
    PlayerStatisticChanged, SjmlError, SjmlResult, StableDecode, StatisticFamily,
};

static EXPECTED_PLAYER_HASH: AtomicU32 = AtomicU32::new(0);
static EXPECTED_TYPE_HASH: AtomicU32 = AtomicU32::new(0);
static EXPECTED_STAT_HASH: AtomicU32 = AtomicU32::new(0);
static EXPECTED_PREVIOUS: AtomicI32 = AtomicI32::new(0);
static EXPECTED_CURRENT: AtomicI32 = AtomicI32::new(0);
static SAW_STATISTIC_CHANGE: AtomicBool = AtomicBool::new(false);

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("player.statistic_changed", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.statistic_changed")]
fn observe_statistic_change(event: PlayerStatisticChanged) -> SjmlResult<()> {
    if string_hash(&event.player_id) == EXPECTED_PLAYER_HASH.load(Ordering::Relaxed)
        && string_hash(&event.statistic_type_id) == EXPECTED_TYPE_HASH.load(Ordering::Relaxed)
        && string_hash(&event.statistic_id) == EXPECTED_STAT_HASH.load(Ordering::Relaxed)
        && event.previous_value == EXPECTED_PREVIOUS.load(Ordering::Relaxed)
        && event.current_value == EXPECTED_CURRENT.load(Ordering::Relaxed)
    {
        SAW_STATISTIC_CHANGE.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct PlayerId {
    uuid: String,
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct StatisticChangeTestArguments {
    operation: String,
    player_id: String,
    statistic_type_id: String,
    statistic_id: String,
    previous_value: i32,
    current_value: i32,
}

#[command(name = "player.statistics.observe")]
fn observe_statistic_change_test(arguments: StatisticChangeTestArguments) -> SjmlResult<String> {
    match arguments.operation.as_str() {
        "arm" => {
            EXPECTED_PLAYER_HASH.store(string_hash(&arguments.player_id), Ordering::Relaxed);
            EXPECTED_TYPE_HASH.store(string_hash(&arguments.statistic_type_id), Ordering::Relaxed);
            EXPECTED_STAT_HASH.store(string_hash(&arguments.statistic_id), Ordering::Relaxed);
            EXPECTED_PREVIOUS.store(arguments.previous_value, Ordering::Relaxed);
            EXPECTED_CURRENT.store(arguments.current_value, Ordering::Relaxed);
            SAW_STATISTIC_CHANGE.store(false, Ordering::Relaxed);
            Ok("player statistic change observer armed".to_owned())
        }
        "check" if SAW_STATISTIC_CHANGE.swap(false, Ordering::Relaxed) => {
            Ok("guest received committed player.statistic_changed snapshot".to_owned())
        }
        "check" => Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "guest did not receive the expected player.statistic_changed snapshot",
        )),
        _ => Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "operation must be either arm or check",
        )),
    }
}

fn string_hash(value: &str) -> u32 {
    value
        .bytes()
        .fold(2_166_136_261, |hash, byte| {
            (hash ^ u32::from(byte)).wrapping_mul(16_777_619)
        })
        .max(1)
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

#[command(name = "player.statistics.registry.increment")]
fn check_registry_statistics(input: PlayerId) -> SjmlResult<String> {
    let handle = player_handle(&input.uuid)?;
    increment_and_verify(handle, StatisticFamily::BlockMined, "minecraft:stone")?;
    for (family, registry_id) in [
        (StatisticFamily::ItemUsed, "minecraft:diamond"),
        (StatisticFamily::ItemBroken, "minecraft:diamond"),
        (StatisticFamily::ItemCrafted, "minecraft:diamond"),
        (StatisticFamily::EntityKilled, "minecraft:zombie"),
        (StatisticFamily::EntityKilledBy, "minecraft:zombie"),
    ] {
        increment_and_verify(handle, family, registry_id)?;
    }
    Ok("registry-backed statistics incremented".to_owned())
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
