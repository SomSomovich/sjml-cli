use sjml::{player, ErrorCode, PlayerHandle};

#[test]
fn action_bar_enforces_literal_text_limits_before_host_access() {
    let handle = PlayerHandle::new(1, 2, 3, 4).unwrap();

    for invalid in ["", " \t\n", "line\nbreak", "bell\u{7}"] {
        assert_eq!(
            player::send_action_bar(handle, invalid).unwrap_err().code(),
            ErrorCode::InvalidArgument,
            "expected local rejection for {invalid:?}"
        );
    }
    assert_eq!(
        player::send_action_bar(handle, &"é".repeat(513))
            .unwrap_err()
            .code(),
        ErrorCode::PayloadTooLarge,
        "the UTF-8 byte limit applies even when the scalar limit is not exceeded"
    );
    assert_eq!(
        player::send_action_bar(handle, &"x".repeat(257))
            .unwrap_err()
            .code(),
        ErrorCode::PayloadTooLarge
    );
}

#[test]
fn valid_unicode_and_json_shaped_text_are_passed_to_the_wasm_host_as_literal_text() {
    let handle = PlayerHandle::new(1, 2, 3, 4).unwrap();
    for text in ["Привет ✨", r#"{"text":"literal"}"#] {
        assert_eq!(
            player::send_action_bar(handle, text).unwrap_err().code(),
            ErrorCode::UnknownErrorCode,
            "valid text reaches the wasm-only host boundary on native targets"
        );
    }
}
