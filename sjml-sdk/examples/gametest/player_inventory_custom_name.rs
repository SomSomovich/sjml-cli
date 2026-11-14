use sjml::{
    command, player, player_inventory, ErrorCode, PlayerHandle, SjmlError, SjmlResult, StableDecode,
};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct PlayerInventoryCustomNameArguments {
    player_uuid: String,
    action: String,
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

#[command(name = "player.inventory.custom_name")]
fn update_player_inventory_item_custom_name(
    arguments: PlayerInventoryCustomNameArguments,
) -> SjmlResult<String> {
    let player = find_player(&arguments.player_uuid)?;
    match arguments.action.as_str() {
        "set" => {
            player_inventory::set_item_custom_name(player, 1, "Café 🪨 §cLiteral")?;
            Ok("set the literal player inventory item custom name".to_owned())
        }
        "clear" => {
            player_inventory::clear_item_custom_name(player, 1)?;
            Ok("cleared the player inventory item custom name".to_owned())
        }
        "glint_force" => {
            player_inventory::set_item_glint_override(player, 1, true)?;
            Ok("forced foil on the player inventory item".to_owned())
        }
        "glint_suppress" => {
            player_inventory::set_item_glint_override(player, 1, false)?;
            Ok("suppressed foil on the player inventory item".to_owned())
        }
        "glint_clear" => {
            player_inventory::clear_item_glint_override(player, 1)?;
            Ok("restored vanilla foil behavior on the player inventory item".to_owned())
        }
        _ => Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "action must be set, clear, glint_force, glint_suppress, or glint_clear",
        )),
    }
}
