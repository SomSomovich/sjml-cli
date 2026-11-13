use sjml::{
    scoreboard, DisplaySlot, ErrorCode, DISPLAY_SLOT_COUNT, MAX_OWNED_OBJECTIVE_DISPLAY_BYTES,
    MAX_OWNED_OBJECTIVE_DISPLAY_CODE_POINTS, MAX_OWNED_OBJECTIVE_ID_BYTES,
    MAX_SCOREBOARD_ENTRY_BYTES, MAX_SCOREBOARD_OBJECTIVE_BYTES, MAX_SCORE_VALUE,
};

#[test]
fn scoreboard_limits_are_explicit_and_match_the_host_contract() {
    assert_eq!(MAX_SCOREBOARD_OBJECTIVE_BYTES, 16);
    assert_eq!(MAX_SCOREBOARD_ENTRY_BYTES, 40);
    assert_eq!(MAX_SCORE_VALUE, 1_000_000_000);
    assert_eq!(DISPLAY_SLOT_COUNT, 19);
    let slots = [
        DisplaySlot::List,
        DisplaySlot::Sidebar,
        DisplaySlot::BelowName,
        DisplaySlot::TeamBlack,
        DisplaySlot::TeamDarkBlue,
        DisplaySlot::TeamDarkGreen,
        DisplaySlot::TeamDarkAqua,
        DisplaySlot::TeamDarkRed,
        DisplaySlot::TeamDarkPurple,
        DisplaySlot::TeamGold,
        DisplaySlot::TeamGray,
        DisplaySlot::TeamDarkGray,
        DisplaySlot::TeamBlue,
        DisplaySlot::TeamGreen,
        DisplaySlot::TeamAqua,
        DisplaySlot::TeamRed,
        DisplaySlot::TeamLightPurple,
        DisplaySlot::TeamYellow,
        DisplaySlot::TeamWhite,
    ];
    assert_eq!(
        slots.map(|slot| slot as u32),
        std::array::from_fn(|index| index as u32)
    );
}

#[test]
fn owned_objective_ids_and_display_names_are_bounded_before_host_access() {
    assert_eq!(MAX_OWNED_OBJECTIVE_ID_BYTES, 64);
    assert_eq!(MAX_OWNED_OBJECTIVE_DISPLAY_BYTES, 128);
    assert_eq!(MAX_OWNED_OBJECTIVE_DISPLAY_CODE_POINTS, 64);

    let long_id = "x".repeat(MAX_OWNED_OBJECTIVE_ID_BYTES + 1);
    for id in ["", "Upper", "bad id", "_start", long_id.as_str()] {
        assert_eq!(
            scoreboard::create_objective(id, "Objective")
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            scoreboard::remove_objective(id).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }
    let long_display = "x".repeat(MAX_OWNED_OBJECTIVE_DISPLAY_BYTES + 1);
    for display_name in ["", "line\nbreak", long_display.as_str()] {
        assert_eq!(
            scoreboard::create_objective("score", display_name)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }

    assert_eq!(
        scoreboard::create_objective("score", "Valid")
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        scoreboard::remove_objective("score").unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        scoreboard::objective_score("score", "#hidden")
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        scoreboard::set_objective_score("score", "Alice", MAX_SCORE_VALUE + 1)
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        scoreboard::set_objective_score("score", "Alice", 1)
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        scoreboard::set_objective_display_slot("score", DisplaySlot::Sidebar)
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
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
    for objective in ["", "bad objective", "objective_name_that_is_long"] {
        assert_eq!(
            scoreboard::set_display_slot(objective, DisplaySlot::Sidebar)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }
    assert_eq!(
        scoreboard::set_display_slot("kills", DisplaySlot::Sidebar)
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        scoreboard::clear_display_slot(DisplaySlot::Sidebar)
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
}
