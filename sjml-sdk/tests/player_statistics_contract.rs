use sjml::{
    player, player_statistics, ErrorCode, PlayerCustomStat, PlayerHandle, PlayerStatistic,
    StatisticFamily,
};

#[test]
fn vanilla_custom_stat_enum_maps_to_the_registered_1_21_1_ids() {
    assert_eq!(PlayerCustomStat::Deaths.id(), "minecraft:deaths");
    assert_eq!(PlayerCustomStat::PlayTime.id(), "minecraft:play_time");
    assert_eq!(
        PlayerCustomStat::WalkOnWaterOneCm.id(),
        "minecraft:walk_on_water_one_cm"
    );
    assert_eq!(
        PlayerCustomStat::InteractWithBrewingStand.id(),
        "minecraft:interact_with_brewingstand"
    );
    assert_eq!(
        PlayerCustomStat::InteractWithSmithingTable.id(),
        "minecraft:interact_with_smithing_table"
    );
}

#[test]
fn statistic_increment_amount_is_validated_before_host_access() {
    let handle = PlayerHandle::new(1, 2, 3, 4).unwrap();
    assert_eq!(player::MAX_PLAYER_STATISTIC_INCREMENT, 1_000_000);
    for amount in [0, player::MAX_PLAYER_STATISTIC_INCREMENT + 1, u32::MAX] {
        assert_eq!(
            player::increment_custom_stat(handle, PlayerCustomStat::Deaths, amount)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument,
            "invalid increment must fail before host access: {amount}"
        );
    }
}

#[test]
fn exact_statistic_values_reject_unsigned_overflow_before_host_access() {
    let handle = PlayerHandle::new(1, 2, 3, 4).unwrap();
    assert_eq!(player::MAX_PLAYER_STATISTIC_VALUE, i32::MAX as u32);
    assert_eq!(
        player::set_custom_stat(
            handle,
            PlayerCustomStat::Deaths,
            player::MAX_PLAYER_STATISTIC_VALUE + 1
        )
        .unwrap_err()
        .code(),
        ErrorCode::InvalidArgument
    );

    let item_stat = PlayerStatistic::new(StatisticFamily::ItemUsed, "minecraft:diamond").unwrap();
    assert_eq!(
        player_statistics::set_value(handle, &item_stat, player_statistics::MAX_VALUE + 1)
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
}

#[test]
fn valid_custom_stat_operations_fail_closed_without_a_wasm_host() {
    let handle = PlayerHandle::new(1, 2, 3, 4).unwrap();
    assert_eq!(
        player::custom_stat(handle, PlayerCustomStat::Deaths)
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        player::increment_custom_stat(handle, PlayerCustomStat::Deaths, 1)
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        player::set_custom_stat(handle, PlayerCustomStat::Deaths, 1)
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
    let item_stat = PlayerStatistic::new(StatisticFamily::ItemUsed, "minecraft:diamond").unwrap();
    assert_eq!(
        player_statistics::set_value(handle, &item_stat, 1)
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
}
