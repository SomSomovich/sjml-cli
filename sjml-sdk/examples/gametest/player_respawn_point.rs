use sjml::{command, player, ErrorCode, PlayerRespawnPoint, SjmlError, SjmlResult, StableDecode};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct RespawnPointArguments {
    action: String,
    uuid: String,
    x: i32,
    y: i32,
    z: i32,
    dimension_id: String,
    angle: String,
    forced: String,
}

fn player_handle(uuid: &str) -> SjmlResult<sjml::PlayerHandle> {
    for handle in player::online()? {
        if player::snapshot(handle)?.uuid == uuid {
            return Ok(handle);
        }
    }
    Err(SjmlError::new(
        ErrorCode::InvalidArgument,
        "GameTest player was not present in the live player list",
    ))
}

fn expected_point(arguments: &RespawnPointArguments) -> SjmlResult<PlayerRespawnPoint> {
    let angle = arguments.angle.parse::<f32>().map_err(|_| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "respawn angle must be a finite number",
        )
    })?;
    let forced = arguments
        .forced
        .parse::<bool>()
        .map_err(|_| SjmlError::new(ErrorCode::InvalidArgument, "forced must be true or false"))?;
    PlayerRespawnPoint::new(
        arguments.x,
        arguments.y,
        arguments.z,
        arguments.dimension_id.clone(),
        angle,
        forced,
    )
}

#[command(name = "player.respawn_point.control")]
fn control_respawn_point(arguments: RespawnPointArguments) -> SjmlResult<String> {
    let handle = player_handle(&arguments.uuid)?;
    match arguments.action.as_str() {
        "set" => {
            player::set_respawn_point(handle, expected_point(&arguments)?)?;
            Ok("guest set the player's vanilla respawn point".to_owned())
        }
        "check" => {
            let actual = player::respawn_point(handle)?.ok_or_else(|| {
                SjmlError::new(
                    ErrorCode::InvalidArgument,
                    "guest did not find a configured respawn point",
                )
            })?;
            let expected = expected_point(&arguments)?;
            if actual.x == expected.x
                && actual.y == expected.y
                && actual.z == expected.z
                && actual.dimension_id == expected.dimension_id
                && actual.angle.to_bits() == expected.angle.to_bits()
                && actual.forced == expected.forced
            {
                Ok("guest read the configured player respawn point".to_owned())
            } else {
                Err(SjmlError::new(
                    ErrorCode::InvalidArgument,
                    "guest player respawn snapshot did not match the expected position, dimension, angle, and forced flag",
                ))
            }
        }
        "clear" => {
            player::clear_respawn_point(handle)?;
            Ok("guest cleared the player's vanilla respawn point".to_owned())
        }
        _ => Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "respawn operation must be set, check, or clear",
        )),
    }
}
