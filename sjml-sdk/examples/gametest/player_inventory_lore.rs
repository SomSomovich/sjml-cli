use sjml::{
    command, player, player_inventory, ErrorCode, PlayerHandle, SjmlError, SjmlResult, StableDecode,
};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct PlayerInventoryLoreArguments {
    player_uuid: String,
}

fn find_player(uuid: &str) -> SjmlResult<PlayerHandle> {
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

#[command(name = "player.inventory.lore.set")]
fn check_and_set_player_inventory_lore(
    arguments: PlayerInventoryLoreArguments,
) -> SjmlResult<String> {
    let player = find_player(&arguments.player_uuid)?;
    let slot = 1;
    let initial = player_inventory::read_lore(player, slot)?;
    if initial != ["previous lore"] {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player lore fixture did not expose its pre-existing plain text line",
        ));
    }
    if !player_inventory::read_lore(player, 2)?.is_empty() {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "an empty inventory slot must read as having no lore",
        ));
    }
    match player_inventory::set_lore(player, 2, &["must not create an item".to_owned()]) {
        Err(error) if error.code() == ErrorCode::InvalidArgument => {}
        _ => {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "setting lore must reject an empty inventory slot",
            ));
        }
    }
    match player_inventory::clear_lore(player, 2) {
        Err(error) if error.code() == ErrorCode::InvalidArgument => {}
        _ => {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "clearing lore must reject an empty inventory slot",
            ));
        }
    }

    let lines = vec![
        "Café 🪨".to_owned(),
        "§cstill literal".to_owned(),
        String::new(),
    ];
    player_inventory::set_lore(player, slot, &lines)?;
    if player_inventory::read_lore(player, slot)? != lines {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player lore lines did not round-trip as literal text",
        ));
    }

    for invalid in [
        Vec::new(),
        vec!["line\nbreak".to_owned()],
        vec!["é".repeat(129)],
        vec!["line".to_owned(); 9],
    ] {
        match player_inventory::set_lore(player, slot, &invalid) {
            Err(error)
                if matches!(
                    error.code(),
                    ErrorCode::InvalidArgument | ErrorCode::PayloadTooLarge
                ) => {}
            _ => {
                return Err(SjmlError::new(
                    ErrorCode::InvalidArgument,
                    "invalid player lore was not rejected before changing the stack",
                ));
            }
        }
    }
    if player_inventory::read_lore(player, slot)? != lines {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "rejected lore input changed the existing lore",
        ));
    }

    Ok("player inventory lore read, set, and validation succeeded".to_owned())
}

#[command(name = "player.inventory.lore.clear")]
fn clear_player_inventory_lore(arguments: PlayerInventoryLoreArguments) -> SjmlResult<String> {
    let player = find_player(&arguments.player_uuid)?;
    let slot = 1;
    player_inventory::clear_lore(player, slot)?;
    if !player_inventory::read_lore(player, slot)?.is_empty() {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "clearing player lore left lines on the stack",
        ));
    }
    Ok("player inventory lore clear succeeded".to_owned())
}
