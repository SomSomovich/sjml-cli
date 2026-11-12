use sjml::events::ClientScreenAction;
use sjml::{
    client_screen, event, from_bytes, to_bytes, ClientScreenButton, ClientScreenDefinition,
    ErrorCode, SjmlResult,
};

#[event(name = "client.screen.action")]
fn on_client_screen_action(action: ClientScreenAction) -> SjmlResult<()> {
    assert_eq!(action.screen_id, "demo:confirmation");
    assert_eq!(action.button_id, "accept");
    Ok(())
}

#[test]
fn screen_definition_accepts_documented_bounds_and_native_open_fails_closed() {
    let definition = ClientScreenDefinition::new(
        "demo:confirmation",
        "Confirm",
        vec![String::new(), "Do you want to continue?".to_owned()],
        vec![ClientScreenButton::new("accept", "Accept").unwrap()],
    )
    .unwrap();
    assert_eq!(definition.id, "demo:confirmation");
    assert_eq!(definition.body[0], "");

    #[cfg(not(target_arch = "wasm32"))]
    assert_eq!(
        client_screen::open(&definition).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
}

#[test]
fn screen_identifiers_and_text_enforce_their_byte_limits_and_control_rules() {
    let button = || ClientScreenButton::new("ok", "OK").unwrap();
    for id in ["", "Upper", "with space", "bad?", "é"] {
        assert_eq!(
            ClientScreenDefinition::new(id, "Title", Vec::new(), vec![button()])
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument,
            "screen ID must reject {id:?}"
        );
    }
    assert_eq!(
        ClientScreenDefinition::new("a".repeat(65), "Title", Vec::new(), vec![button()])
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        ClientScreenDefinition::new("demo:ok", "", Vec::new(), vec![button()])
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        ClientScreenDefinition::new("demo:ok", "x".repeat(129), Vec::new(), vec![button()])
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        ClientScreenDefinition::new("demo:ok", "Title\n", Vec::new(), vec![button()])
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        ClientScreenDefinition::new("demo:ok", "Title", vec!["x".repeat(161)], vec![button()],)
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        ClientScreenDefinition::new(
            "demo:ok",
            "Title",
            vec!["line\twith control".to_owned()],
            vec![button()],
        )
        .unwrap_err()
        .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        ClientScreenButton::new("ok", "line\nwith control")
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
}

#[test]
fn screens_bound_line_and_button_counts_and_require_unique_button_ids() {
    let button = || ClientScreenButton::new("ok", "OK").unwrap();
    assert_eq!(
        ClientScreenDefinition::new("demo:ok", "Title", vec![String::new(); 7], vec![button()])
            .unwrap_err()
            .code(),
        ErrorCode::PayloadTooLarge
    );
    assert_eq!(
        ClientScreenDefinition::new("demo:ok", "Title", Vec::new(), Vec::new())
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        ClientScreenDefinition::new("demo:ok", "Title", Vec::new(), vec![button(); 5])
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        ClientScreenDefinition::new("demo:ok", "Title", Vec::new(), vec![button(), button()],)
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        ClientScreenButton::new("bad?", "Label").unwrap_err().code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        ClientScreenButton::new("ok", "").unwrap_err().code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        ClientScreenButton::new("ok", "x".repeat(97))
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
}

#[test]
fn open_revalidates_public_fields_and_rejects_controls_before_host_access() {
    let mut definition = ClientScreenDefinition::new(
        "demo:ok",
        "Title",
        Vec::new(),
        vec![ClientScreenButton::new("ok", "OK").unwrap()],
    )
    .unwrap();
    definition.body.push("invalid\rline".to_owned());
    assert_eq!(
        client_screen::open(&definition).unwrap_err().code(),
        ErrorCode::InvalidArgument
    );
}

#[test]
fn client_screen_action_uses_the_v1_two_string_event_payload_and_callback() {
    let action = ClientScreenAction {
        screen_id: "demo:confirmation".to_owned(),
        button_id: "accept".to_owned(),
    };
    let mut expected = 1_u32.to_le_bytes().to_vec();
    append_string(&mut expected, "demo:confirmation");
    append_string(&mut expected, "accept");
    assert_eq!(to_bytes(&action).unwrap(), expected);
    assert_eq!(from_bytes::<ClientScreenAction>(&expected).unwrap(), action);
    assert_eq!(on_client_screen_action(action), Ok(()));
}

fn append_string(target: &mut Vec<u8>, value: &str) {
    target.extend_from_slice(&(value.len() as u32).to_le_bytes());
    target.extend_from_slice(value.as_bytes());
}
