use sjml::events::{
    ClientScreenAction, ClientScreenFormSubmit, ClientScreenFormValue,
    ClientScreenMultiSelectFormSubmit,
};
use sjml::{
    client_screen, event, from_bytes, to_bytes, ClientScreenButton, ClientScreenDefinition,
    ClientScreenFormDefinition, ClientScreenListFormDefinition,
    ClientScreenMultiSelectFormDefinition, ClientScreenMultiSelectionList,
    ClientScreenSelectionList, ClientScreenSelectionOption, ClientScreenTextField, ErrorCode,
    SjmlResult,
};

#[event(name = "client.screen.action")]
fn on_client_screen_action(action: ClientScreenAction) -> SjmlResult<()> {
    assert_eq!(action.screen_id, "demo:confirmation");
    assert_eq!(action.button_id, "accept");
    Ok(())
}

#[event(name = "client.screen.form_submit")]
fn on_client_screen_form_submit(form: ClientScreenFormSubmit) -> SjmlResult<()> {
    assert_eq!(form.screen_id, "demo:profile");
    assert_eq!(form.button_id, "save");
    assert_eq!(
        form.fields,
        vec![ClientScreenFormValue {
            id: "name".to_owned(),
            value: "Ada".to_owned(),
        }]
    );
    Ok(())
}

#[event(name = "client.screen.multi_select_form_submit")]
fn on_client_screen_multi_select_form_submit(
    form: ClientScreenMultiSelectFormSubmit,
) -> SjmlResult<()> {
    assert_eq!(form.screen_id, "demo:features");
    assert_eq!(form.list_id, "features");
    assert_eq!(form.selected_option_ids, vec!["chat", "homes"]);
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

#[test]
fn form_definition_validates_field_bounds_and_open_fails_closed_on_native_targets() {
    let screen = || {
        ClientScreenDefinition::new(
            "demo:profile",
            "Profile",
            Vec::new(),
            vec![ClientScreenButton::new("save", "Save").unwrap()],
        )
        .unwrap()
    };
    let field = || ClientScreenTextField::new("name", "Name", "Ada", 64).unwrap();
    let definition = ClientScreenFormDefinition::new(screen(), vec![field()]).unwrap();
    assert_eq!(definition.fields[0].initial_value, "Ada");

    #[cfg(not(target_arch = "wasm32"))]
    assert_eq!(
        client_screen::open_form(&definition).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );

    for (id, label, value, max_bytes) in [
        ("Upper", "Name", "", 32),
        ("name", "", "", 32),
        ("name", "Name", "é", 1),
        ("name", "Name", "x", 0),
        ("name", "Name", "x", 129),
        ("name", "Name", "a\nb", 8),
    ] {
        assert_eq!(
            ClientScreenTextField::new(id, label, value, max_bytes)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }

    assert_eq!(
        ClientScreenTextField::new("name", "Name", "x".repeat(129), 128)
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        ClientScreenFormDefinition::new(screen(), Vec::new())
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        ClientScreenFormDefinition::new(
            screen(),
            (0..5)
                .map(|index| {
                    ClientScreenTextField::new(format!("field{index}"), "Field", "", 32).unwrap()
                })
                .collect(),
        )
        .unwrap_err()
        .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        ClientScreenFormDefinition::new(
            screen(),
            vec![ClientScreenTextField::new("save", "Name", "", 32).unwrap()],
        )
        .unwrap_err()
        .code(),
        ErrorCode::InvalidArgument
    );
}

#[test]
fn form_submit_callback_uses_stable_schema_v1_and_preserves_field_values() {
    let form = ClientScreenFormSubmit {
        screen_id: "demo:profile".to_owned(),
        button_id: "save".to_owned(),
        fields: vec![ClientScreenFormValue {
            id: "name".to_owned(),
            value: "Ada".to_owned(),
        }],
    };
    let mut expected = 1_u32.to_le_bytes().to_vec();
    append_string(&mut expected, "demo:profile");
    append_string(&mut expected, "save");
    expected.extend_from_slice(&1_u32.to_le_bytes());
    expected.extend_from_slice(&1_u32.to_le_bytes());
    append_string(&mut expected, "name");
    append_string(&mut expected, "Ada");
    assert_eq!(to_bytes(&form).unwrap(), expected);
    assert_eq!(
        from_bytes::<ClientScreenFormSubmit>(&expected).unwrap(),
        form
    );
    assert_eq!(on_client_screen_form_submit(form), Ok(()));
}

#[test]
fn list_form_accepts_zero_to_four_text_fields_and_fails_closed_on_native_targets() {
    let screen = || {
        ClientScreenDefinition::new(
            "demo:settings",
            "Settings",
            Vec::new(),
            vec![ClientScreenButton::new("apply", "Apply").unwrap()],
        )
        .unwrap()
    };
    let list = || {
        ClientScreenSelectionList::new(
            "mode",
            "Mode",
            vec![
                ClientScreenSelectionOption::new("easy", "Easy").unwrap(),
                ClientScreenSelectionOption::new("hard", "Hard").unwrap(),
            ],
            None,
        )
        .unwrap()
    };
    let definition = ClientScreenListFormDefinition::new(screen(), Vec::new(), list()).unwrap();
    #[cfg(not(target_arch = "wasm32"))]
    assert_eq!(
        client_screen::open_list_form(&definition)
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );

    assert_eq!(
        ClientScreenSelectionList::new(
            "mode",
            "Mode",
            vec![ClientScreenSelectionOption::new("easy", "Easy").unwrap()],
            Some("unknown".to_owned()),
        )
        .unwrap_err()
        .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        ClientScreenSelectionList::new(
            "mode",
            "Mode",
            (0..33)
                .map(|index| {
                    ClientScreenSelectionOption::new(format!("option{index}"), "Choice").unwrap()
                })
                .collect(),
            None,
        )
        .unwrap_err()
        .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        ClientScreenListFormDefinition::new(
            screen(),
            (0..5)
                .map(|index| {
                    ClientScreenTextField::new(format!("field{index}"), "Field", "", 32).unwrap()
                })
                .collect(),
            list(),
        )
        .unwrap_err()
        .code(),
        ErrorCode::InvalidArgument
    );
}

#[test]
fn form_submit_can_encode_selection_as_a_generic_field_value() {
    let submit = ClientScreenFormSubmit {
        screen_id: "demo:settings".to_owned(),
        button_id: "apply".to_owned(),
        fields: vec![ClientScreenFormValue {
            id: "mode".to_owned(),
            value: String::new(),
        }],
    };
    assert_eq!(
        from_bytes::<ClientScreenFormSubmit>(&to_bytes(&submit).unwrap()).unwrap(),
        submit
    );
}

#[test]
fn multi_select_submit_callback_uses_schema_v6_and_returns_declared_option_ids() {
    let submit = ClientScreenMultiSelectFormSubmit {
        screen_id: "demo:features".to_owned(),
        button_id: "save".to_owned(),
        fields: vec![ClientScreenFormValue {
            id: "name".to_owned(),
            value: "Ada".to_owned(),
        }],
        list_id: "features".to_owned(),
        selected_option_ids: vec!["chat".to_owned(), "homes".to_owned()],
    };
    let mut expected = 6_u32.to_le_bytes().to_vec();
    append_string(&mut expected, "demo:features");
    append_string(&mut expected, "save");
    expected.extend_from_slice(&1_u32.to_le_bytes());
    expected.extend_from_slice(&1_u32.to_le_bytes());
    append_string(&mut expected, "name");
    append_string(&mut expected, "Ada");
    append_string(&mut expected, "features");
    expected.extend_from_slice(&2_u32.to_le_bytes());
    append_string(&mut expected, "chat");
    append_string(&mut expected, "homes");
    assert_eq!(to_bytes(&submit).unwrap(), expected);
    assert_eq!(
        from_bytes::<ClientScreenMultiSelectFormSubmit>(&expected).unwrap(),
        submit
    );
    assert_eq!(on_client_screen_multi_select_form_submit(submit), Ok(()));
}

#[test]
fn multi_select_form_accepts_zero_text_fields_and_fails_closed_on_native_targets() {
    let screen = ClientScreenDefinition::new(
        "demo:features",
        "Features",
        Vec::new(),
        vec![ClientScreenButton::new("save", "Save").unwrap()],
    )
    .unwrap();
    let list = ClientScreenMultiSelectionList::new(
        "features",
        "Features",
        vec![ClientScreenSelectionOption::new("chat", "Chat").unwrap()],
        vec!["chat".to_owned()],
    )
    .unwrap();
    let form = ClientScreenMultiSelectFormDefinition::new(screen, Vec::new(), list).unwrap();
    #[cfg(not(target_arch = "wasm32"))]
    assert_eq!(
        client_screen::open_multi_select_form(&form)
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
}

fn append_string(target: &mut Vec<u8>, value: &str) {
    target.extend_from_slice(&(value.len() as u32).to_le_bytes());
    target.extend_from_slice(value.as_bytes());
}
