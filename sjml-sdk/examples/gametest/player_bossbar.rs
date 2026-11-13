use sjml::{command, ErrorCode, SjmlError, SjmlResult, StableDecode};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct BossBarArguments {
    first_player_id: String,
    second_player_id: String,
}

/// Uses two real player handles to check ownership, then exercises a complete vanilla lifecycle.
#[command(name = "player.bossbar.check")]
fn boss_bar_check(arguments: BossBarArguments) -> SjmlResult<String> {
    let mut first = None;
    let mut second = None;
    for handle in sjml::player::online()? {
        let snapshot = sjml::player::snapshot(handle)?;
        if snapshot.uuid == arguments.first_player_id {
            first = Some(handle);
        }
        if snapshot.uuid == arguments.second_player_id {
            second = Some(handle);
        }
    }
    let first = first.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidHandle,
            "first GameTest player was not online",
        )
    })?;
    let second = second.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidHandle,
            "second GameTest player was not online",
        )
    })?;

    let initial = sjml::player_bossbar::BossBarDefinition::new(
        "GameTest boss bar",
        0.25,
        sjml::player_bossbar::BossBarColor::Blue,
        sjml::player_bossbar::BossBarOverlay::Progress,
        true,
    )?;
    let id = sjml::player_bossbar::create(first, &initial)?;

    let updated = sjml::player_bossbar::BossBarDefinition::new(
        "Updated GameTest boss bar",
        0.75,
        sjml::player_bossbar::BossBarColor::Green,
        sjml::player_bossbar::BossBarOverlay::Notched10,
        true,
    )?;
    match sjml::player_bossbar::update(second, id, &updated) {
        Err(error) if error.code() == ErrorCode::InvalidHandle => {}
        Err(error) => return Err(error),
        Ok(()) => {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "a boss bar must reject a different live player handle",
            ));
        }
    }
    sjml::player_bossbar::update(first, id, &updated)?;
    sjml::player_bossbar::remove(first, id)?;
    match sjml::player_bossbar::remove(first, id) {
        Err(error) if error.code() == ErrorCode::InvalidHandle => {}
        Err(error) => return Err(error),
        Ok(()) => {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "a removed boss bar must not remain available",
            ));
        }
    }
    Ok("created, updated, isolated, and removed a vanilla boss bar".to_owned())
}
