use sjml::{command, ErrorCode, SjmlError, SjmlResult, StableDecode};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct ScoreboardArguments {
    objective: String,
    entry: String,
    expected_value: i32,
}

#[command(name = "scoreboard.check")]
fn read_and_write_existing_score(arguments: ScoreboardArguments) -> SjmlResult<String> {
    let initial = sjml::scoreboard::score(&arguments.objective, &arguments.entry)?;
    if initial != Some(7) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "scoreboard GameTest must create an existing dummy score initialized to seven",
        ));
    }
    sjml::scoreboard::set_score(
        &arguments.objective,
        &arguments.entry,
        arguments.expected_value,
    )?;
    if sjml::scoreboard::score(&arguments.objective, &arguments.entry)?
        != Some(arguments.expected_value)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "scoreboard read after write did not return the requested value",
        ));
    }
    Ok("existing scoreboard objective read and score write succeeded".to_owned())
}
