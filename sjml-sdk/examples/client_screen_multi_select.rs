#![cfg_attr(
    not(all(target_arch = "wasm32", target_os = "unknown")),
    allow(dead_code)
)]

use sjml::prelude::*;

#[client_command(name = "features.open")]
fn open_feature_settings(category: String) -> SjmlResult<String> {
    let screen = ClientScreenDefinition::new(
        "demo:features",
        "Feature selection",
        vec![format!("Choose {category} features to enable.")],
        vec![ClientScreenButton::new("save", "Save").unwrap()],
    )?;
    let list = ClientScreenMultiSelectionList::new(
        "features",
        "Enabled features",
        vec![
            ClientScreenSelectionOption::new("chat", "Chat")?,
            ClientScreenSelectionOption::new("homes", "Homes")?,
            ClientScreenSelectionOption::new("claims", "Claims")?,
        ],
        vec!["chat".to_owned()],
    )?;
    client_screen::open_multi_select_form(&ClientScreenMultiSelectFormDefinition::new(
        screen,
        Vec::new(),
        list,
    )?)?;
    Ok("Opened feature settings".to_owned())
}

#[event(name = "client.screen.multi_select_form_submit")]
fn save_feature_settings(event: ClientScreenMultiSelectFormSubmit) -> SjmlResult<()> {
    let selected = event.selected_option_ids.join(", ");
    let _ = host::log(LogLevel::Info, &format!("Selected features: {selected}"));
    Ok(())
}
