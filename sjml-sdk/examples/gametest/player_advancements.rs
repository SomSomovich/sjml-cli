use sjml::{command, player, ErrorCode, SjmlError, SjmlResult, StableDecode};

const ADVANCEMENT_ID: &str = "minecraft:story/root";

#[derive(StableDecode)]
#[sjml(version = 1)]
struct PlayerId {
    uuid: String,
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct AdvancementId {
    uuid: String,
    advancement_id: String,
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

#[command(name = "player.advancement.criterion.check")]
fn check_advancement_criterion(input: AdvancementId) -> SjmlResult<String> {
    let handle = player_handle(&input.uuid)?;
    let before = sjml::player_advancement::progress(handle, &input.advancement_id)?;
    let criterion = before
        .criteria
        .iter()
        .find(|criterion| !criterion.achieved)
        .ok_or_else(|| {
            SjmlError::new(
                ErrorCode::InvalidArgument,
                "advancement must have an unachieved criterion before the guest check",
            )
        })?
        .id
        .clone();

    if !sjml::player_advancement::grant_criterion(handle, &input.advancement_id, &criterion)? {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "granting the unachieved advancement criterion did not change progress",
        ));
    }
    let granted = sjml::player_advancement::progress(handle, &input.advancement_id)?;
    if granted
        .criterion(&criterion)
        .is_none_or(|state| !state.achieved)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "criterion-level advancement grant did not round-trip",
        ));
    }

    if !sjml::player_advancement::revoke_criterion(handle, &input.advancement_id, &criterion)? {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "revoking the achieved advancement criterion did not change progress",
        ));
    }
    let revoked = sjml::player_advancement::progress(handle, &input.advancement_id)?;
    if revoked
        .criterion(&criterion)
        .is_none_or(|state| state.achieved)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "criterion-level advancement revoke did not round-trip",
        ));
    }
    Ok(format!("criterion {criterion} was granted and revoked"))
}
