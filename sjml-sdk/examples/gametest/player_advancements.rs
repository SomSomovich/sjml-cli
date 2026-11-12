use sjml::{command, player, ErrorCode, SjmlError, SjmlResult, StableDecode};

const ADVANCEMENT_ID: &str = "minecraft:story/root";

#[derive(StableDecode)]
#[sjml(version = 1)]
struct PlayerId {
    uuid: String,
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

#[command(name = "player.advancement.control.grant")]
fn grant_story_root(input: PlayerId) -> SjmlResult<String> {
    let changed = player::grant_advancement(player_handle(&input.uuid)?, ADVANCEMENT_ID)?;
    if changed {
        Ok("guest granted the story root advancement".to_owned())
    } else {
        Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "story root progress was already complete before the guest grant",
        ))
    }
}

#[command(name = "player.advancement.control.revoke")]
fn revoke_story_root(input: PlayerId) -> SjmlResult<String> {
    let changed = player::revoke_advancement(player_handle(&input.uuid)?, ADVANCEMENT_ID)?;
    if changed {
        Ok("guest revoked the story root advancement".to_owned())
    } else {
        Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "story root progress was already incomplete before the guest revoke",
        ))
    }
}
