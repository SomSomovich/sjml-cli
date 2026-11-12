use sjml::{
    command, player, player_inventory, ErrorCode, PlayerHandle, SjmlError, SjmlResult, StableDecode,
};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct PlayerInventoryCustomNameArguments {
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

#[command(name = "player.inventory.custom_name.set")]
fn set_player_inventory_item_custom_name(
    arguments: PlayerInventoryCustomNameArguments,
) -> SjmlResult<String> {
    player_inventory::set_item_custom_name(
        find_player(&arguments.player_uuid)?,
        1,
        "Café 🪨 §cLiteral",
    )?;
    Ok("set the literal player inventory item custom name".to_owned())
}

#[command(name = "player.inventory.custom_name.clear")]
fn clear_player_inventory_item_custom_name(
    arguments: PlayerInventoryCustomNameArguments,
) -> SjmlResult<String> {
    player_inventory::clear_item_custom_name(find_player(&arguments.player_uuid)?, 1)?;
    Ok("cleared the player inventory item custom name".to_owned())
}
