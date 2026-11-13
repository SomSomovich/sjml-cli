use sjml::{command, scoreboard, DisplaySlot, ErrorCode, SjmlError, SjmlResult, StableDecode};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct ObjectiveArguments {
    action: String,
    objective_id: String,
    entry: String,
}

#[command(name = "scoreboard.objective")]
fn owned_objective(arguments: ObjectiveArguments) -> SjmlResult<String> {
    match arguments.action.as_str() {
        "check" => {
            scoreboard::create_objective(&arguments.objective_id, "Initial label")?;
            // Repeating creation updates only this module's literal display label.
            scoreboard::create_objective(&arguments.objective_id, "Updated label")?;
            scoreboard::set_objective_score(&arguments.objective_id, &arguments.entry, 47)?;
            if scoreboard::objective_score(&arguments.objective_id, &arguments.entry)? != Some(47) {
                return Err(SjmlError::new(
                    ErrorCode::InvalidArgument,
                    "owned dummy objective score did not round-trip through the host",
                ));
            }
            scoreboard::set_objective_display_slot(&arguments.objective_id, DisplaySlot::Sidebar)?;
            Ok(
                "owned scoreboard objective create, update, score, and display succeeded"
                    .to_owned(),
            )
        }
        "cleanup" => {
            scoreboard::remove_objective(&arguments.objective_id)?;
            Ok("owned scoreboard objective cleanup succeeded".to_owned())
        }
        _ => Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "scoreboard objective action must be 'check' or 'cleanup'",
        )),
    }
}
