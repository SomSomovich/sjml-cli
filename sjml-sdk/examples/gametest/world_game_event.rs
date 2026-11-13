use sjml::{command, world, BlockPos, ErrorCode, SjmlError, SjmlResult, StableDecode};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct GameEventArguments {
    x: i32,
    y: i32,
    z: i32,
    event_id: String,
    expect_rejected: bool,
}

#[command(name = "world.game_event.emit")]
fn emit_loaded_game_event(arguments: GameEventArguments) -> SjmlResult<String> {
    let overworld = world::get("minecraft:overworld")?.ok_or_else(|| {
        SjmlError::new(ErrorCode::InvalidArgument, "the Overworld is unavailable")
    })?;
    let position = BlockPos {
        x: arguments.x,
        y: arguments.y,
        z: arguments.z,
    };
    let result = world::emit_game_event(overworld, position, &arguments.event_id);
    if arguments.expect_rejected {
        return match result {
            Err(error) if error.code() == ErrorCode::InvalidArgument => {
                Ok("unavailable game-event target was rejected".to_owned())
            }
            Err(error) => Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                format!("unavailable target returned unexpected error: {error}"),
            )),
            Ok(()) => Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "an event at an unavailable target was accepted",
            )),
        };
    }
    result?;
    Ok("registered game event was dispatched".to_owned())
}
