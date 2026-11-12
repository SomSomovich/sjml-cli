use sjml::{player, ErrorCode, PlayerHandle};

#[test]
fn player_cooldown_item_ids_and_ticks_are_validated_before_host_access() {
    let handle = PlayerHandle::new(1, 2, 3, 4).unwrap();
    assert_eq!(player::MAX_PLAYER_ITEM_COOLDOWN_TICKS, 72_000);
    assert_eq!(player::MAX_PLAYER_COOLDOWN_ITEM_ID_BYTES, 256);

    for invalid in [
        "",
        "diamond",
        "minecraft:",
        ":diamond",
        "Minecraft:diamond",
        "minecraft:air",
    ] {
        assert_eq!(
            player::has_item_cooldown(handle, invalid)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument,
            "invalid item ID should fail before invoking the host: {invalid}"
        );
        assert_eq!(
            player::set_item_cooldown(handle, invalid, 20)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }
    assert_eq!(
        player::has_item_cooldown(handle, &format!("mod:{}", "x".repeat(254)))
            .unwrap_err()
            .code(),
        ErrorCode::PayloadTooLarge
    );

    for ticks in [0, player::MAX_PLAYER_ITEM_COOLDOWN_TICKS + 1] {
        assert_eq!(
            player::set_item_cooldown(handle, "minecraft:diamond", ticks)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument,
            "invalid duration should fail before invoking the host: {ticks}"
        );
    }
}

#[test]
fn public_player_cooldown_calls_fail_closed_without_a_wasm_host() {
    let handle = PlayerHandle::new(1, 2, 3, 4).unwrap();
    assert_eq!(
        player::has_item_cooldown(handle, "minecraft:diamond")
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        player::set_item_cooldown(handle, "minecraft:diamond", 20)
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        player::clear_item_cooldown(handle, "minecraft:diamond")
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
}
