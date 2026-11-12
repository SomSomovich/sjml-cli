use sjml::{command, world, ErrorCode, SjmlError, SjmlResult, StableDecode, WorldDifficulty};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct SetDifficultyArguments {
    value: i32,
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct NoArguments {}

#[command(name = "world.difficulty.control.set")]
fn set_difficulty(arguments: SetDifficultyArguments) -> SjmlResult<String> {
    let difficulty = WorldDifficulty::try_from(arguments.value)?;
    world::set_difficulty(difficulty)?;
    Ok("guest set the server difficulty".to_owned())
}

#[command(name = "world.difficulty.control.check")]
fn check_hard_difficulty(_: NoArguments) -> SjmlResult<String> {
    if world::difficulty()? == WorldDifficulty::Hard {
        Ok("guest read the hard server difficulty".to_owned())
    } else {
        Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "guest did not observe the hard server difficulty",
        ))
    }
}
