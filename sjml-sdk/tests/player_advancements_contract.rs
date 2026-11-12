use sjml::{player, ErrorCode, PlayerHandle};

#[test]
fn advancement_ids_are_validated_before_the_host_is_called() {
    let handle = PlayerHandle::new(1, 2, 3, 4).unwrap();

    for invalid in [
        "",
        "story/root",
        "minecraft:",
        ":story/root",
        "Minecraft:story/root",
    ] {
        assert_eq!(
            player::grant_advancement(handle, invalid)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument,
            "invalid advancement ID should fail before invoking the host: {invalid}"
        );
        assert_eq!(
            player::revoke_advancement(handle, invalid)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }

    let oversized = format!("minecraft:{}", "x".repeat(247));
    assert_eq!(oversized.len(), player::MAX_PLAYER_ADVANCEMENT_ID_BYTES + 1);
    assert_eq!(
        player::grant_advancement(handle, &oversized)
            .unwrap_err()
            .code(),
        ErrorCode::PayloadTooLarge
    );
}

#[test]
fn valid_advancement_operations_fail_closed_without_a_wasm_host() {
    let handle = PlayerHandle::new(1, 2, 3, 4).unwrap();
    assert_eq!(
        player::grant_advancement(handle, "minecraft:story/root")
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        player::revoke_advancement(handle, "minecraft:story/root")
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
}
