use sjml::{command, ErrorCode, SjmlError, SjmlResult, StableDecode};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct PlayerTabListArguments {
    player_id: String,
}

/// Exercises literal tab-list updates and removal for one live server player.
#[command(name = "player.tab_list.check")]
fn player_tab_list_check(arguments: PlayerTabListArguments) -> SjmlResult<String> {
    let mut player = None;
    for handle in sjml::player::online()? {
        if sjml::player::snapshot(handle)?.uuid == arguments.player_id {
            player = Some(handle);
            break;
        }
    }
    let player = player.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidHandle,
            "GameTest player was not present in the online-player snapshot",
        )
    })?;

    sjml::player::set_tab_list(player, "SJML GameTest header", "literal footer")?;
    sjml::player::set_tab_list(player, "updated header", "")?;
    sjml::player::clear_tab_list(player)?;
    Ok("set, updated, and cleared a composed tab-list contribution".to_owned())
}
