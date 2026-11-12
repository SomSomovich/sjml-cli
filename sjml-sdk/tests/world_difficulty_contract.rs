use sjml::{world, ErrorCode, WorldDifficulty};

#[test]
fn difficulty_abi_values_are_the_four_vanilla_levels() {
    for (abi_value, difficulty) in [
        (0, WorldDifficulty::Peaceful),
        (1, WorldDifficulty::Easy),
        (2, WorldDifficulty::Normal),
        (3, WorldDifficulty::Hard),
    ] {
        assert_eq!(difficulty as i32, abi_value);
        assert_eq!(WorldDifficulty::try_from(abi_value).unwrap(), difficulty);
    }
    for invalid in [-1, 4, i32::MAX] {
        assert_eq!(
            WorldDifficulty::try_from(invalid).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }
}

#[test]
fn server_difficulty_operations_fail_closed_without_a_wasm_host() {
    assert_eq!(
        world::difficulty().unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        world::set_difficulty(WorldDifficulty::Hard)
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
}
