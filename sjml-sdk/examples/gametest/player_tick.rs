use sjml::{
    command, event,
    events::{self, EventPriority, PlayerTick},
    ErrorCode, SjmlError, SjmlResult, StableDecode,
};
use std::sync::Mutex;

static LAST_PLAYER_TICK: Mutex<Option<(String, String, i64)>> = Mutex::new(None);

pub(super) fn initialize() -> SjmlResult<()> {
    *LAST_PLAYER_TICK.lock().map_err(|_| {
        SjmlError::new(
            ErrorCode::UnknownErrorCode,
            "player.tick state lock is poisoned",
        )
    })? = None;
    events::subscribe_with_interval("player.tick", EventPriority::Normal, 1)?;
    Ok(())
}

#[event(name = "player.tick")]
fn observe_player_tick(event: PlayerTick) -> SjmlResult<()> {
    *LAST_PLAYER_TICK.lock().map_err(|_| {
        SjmlError::new(
            ErrorCode::UnknownErrorCode,
            "player.tick state lock is poisoned",
        )
    })? = Some((event.player_id, event.dimension_id, event.tick_count));
    Ok(())
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct PlayerTickCheckArguments {
    player_id: String,
    dimension_id: String,
    tick_count: i32,
}

#[command(name = "player.tick.check")]
fn check_player_tick(arguments: PlayerTickCheckArguments) -> SjmlResult<String> {
    let observed = LAST_PLAYER_TICK.lock().map_err(|_| {
        SjmlError::new(
            ErrorCode::UnknownErrorCode,
            "player.tick state lock is poisoned",
        )
    })?;
    let Some((player_id, dimension_id, tick_count)) = observed.as_ref() else {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "the guest has not observed a player.tick callback",
        ));
    };
    if player_id != &arguments.player_id
        || dimension_id != &arguments.dimension_id
        || *tick_count != i64::from(arguments.tick_count)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            format!(
                "player.tick expected {}/{}/{}, received {}/{}/{}",
                arguments.player_id,
                arguments.dimension_id,
                arguments.tick_count,
                player_id,
                dimension_id,
                tick_count
            ),
        ));
    }
    Ok("player.tick delivered matching player, dimension and tick count".to_owned())
}
