use sjml::{player, ErrorCode, PlayerGameMode, PlayerHandle};

#[test]
fn game_mode_variants_keep_their_stable_abi_values() {
    let variants = [
        PlayerGameMode::Survival,
        PlayerGameMode::Creative,
        PlayerGameMode::Adventure,
        PlayerGameMode::Spectator,
    ];
    for (code, mode) in variants.into_iter().enumerate() {
        assert_eq!(mode as u32, code as u32);
        assert_eq!(PlayerGameMode::try_from(code as u32).unwrap(), mode);
    }

    assert_eq!(
        PlayerGameMode::try_from(4).unwrap_err().code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        PlayerGameMode::try_from(u32::MAX).unwrap_err().code(),
        ErrorCode::InvalidArgument
    );
}

#[test]
fn public_player_game_mode_calls_are_available_and_require_a_wasm_host() {
    let handle = PlayerHandle::new(1, 2, 3, 4).unwrap();
    assert_eq!(
        player::game_mode(handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        player::set_game_mode(handle, PlayerGameMode::Creative)
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
}
