use sjml::{
    scoreboard, ErrorCode, MAX_SCOREBOARD_ENTRY_BYTES, MAX_SCOREBOARD_OBJECTIVE_BYTES,
    MAX_SCORE_VALUE,
};

#[test]
fn scoreboard_limits_are_explicit_and_match_the_host_contract() {
    assert_eq!(MAX_SCOREBOARD_OBJECTIVE_BYTES, 16);
    assert_eq!(MAX_SCOREBOARD_ENTRY_BYTES, 40);
    assert_eq!(MAX_SCORE_VALUE, 1_000_000_000);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn invalid_objectives_entries_and_values_fail_before_host_access() {
    for objective in ["", "bad objective", "objective_name_that_is_long"] {
        assert_eq!(
            scoreboard::score(objective, "Alice").unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }
    let long_entry = "x".repeat(MAX_SCOREBOARD_ENTRY_BYTES + 1);
    for entry in ["", "#hidden", "bad\nentry", long_entry.as_str()] {
        assert_eq!(
            scoreboard::score("kills", entry).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }
    assert_eq!(
        scoreboard::set_score("kills", "Alice", MAX_SCORE_VALUE + 1)
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        scoreboard::set_score("kills", "Alice", 1)
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
}
