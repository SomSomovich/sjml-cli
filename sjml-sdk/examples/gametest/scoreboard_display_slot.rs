use sjml::{command, scoreboard, DisplaySlot, ErrorCode, SjmlError, SjmlResult, StableDecode};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct DisplaySlotArguments {
    action: String,
    objective: String,
}

#[command(name = "scoreboard.display_slot")]
fn control_sidebar_display_slot(arguments: DisplaySlotArguments) -> SjmlResult<String> {
    match arguments.action.as_str() {
        "set" => {
            scoreboard::set_display_slot(&arguments.objective, DisplaySlot::Sidebar)?;
            Ok("existing objective displayed in the sidebar".to_owned())
        }
        "clear" => {
            scoreboard::clear_display_slot(DisplaySlot::Sidebar)?;
            Ok("sidebar display slot cleared".to_owned())
        }
        _ => Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "scoreboard display-slot action must be 'set' or 'clear'",
        )),
    }
}
