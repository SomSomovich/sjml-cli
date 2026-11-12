use sjml::{
    command, player, player_inventory, ErrorCode, PlayerHandle, SjmlError, SjmlResult, StableDecode,
};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct PlayerInventoryEnchantmentArguments {
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

#[command(name = "player.inventory.enchantment")]
fn check_player_inventory_enchantment(
    arguments: PlayerInventoryEnchantmentArguments,
) -> SjmlResult<String> {
    let player = find_player(&arguments.player_uuid)?;
    let sharpness = "minecraft:sharpness";
    let unbreaking = "minecraft:unbreaking";

    if player_inventory::read_enchantment_level(player, 1, sharpness)?.is_some()
        || player_inventory::read_enchantment_level(player, 1, unbreaking)? != Some(2)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player sword enchantment fixture did not start with Unbreaking II only",
        ));
    }

    for (enchantment_id, level) in [
        ("minecraft:efficiency", 1),
        ("minecraft:not_an_enchantment", 1),
        (sharpness, 6),
    ] {
        match player_inventory::set_enchantment_level(player, 1, enchantment_id, level) {
            Err(error) if error.code() == ErrorCode::InvalidArgument => {}
            _ => {
                return Err(SjmlError::new(
                    ErrorCode::InvalidArgument,
                    "inapplicable or out-of-range enchantment was not rejected",
                ));
            }
        }
    }

    player_inventory::set_enchantment_level(player, 1, sharpness, 3)?;
    if player_inventory::read_enchantment_level(player, 1, sharpness)? != Some(3)
        || player_inventory::read_enchantment_level(player, 1, unbreaking)? != Some(2)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "setting one sword enchantment changed its level or another enchantment",
        ));
    }

    player_inventory::remove_enchantment(player, 1, sharpness)?;
    if player_inventory::read_enchantment_level(player, 1, sharpness)?.is_some()
        || player_inventory::read_enchantment_level(player, 1, unbreaking)? != Some(2)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "removing Sharpness changed another enchantment or left Sharpness present",
        ));
    }

    let book = "minecraft:mending";
    if player_inventory::read_enchantment_level(player, 2, book)?.is_some() {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "enchanted book fixture did not start empty",
        ));
    }
    player_inventory::set_enchantment_level(player, 2, book, 1)?;
    if player_inventory::read_enchantment_level(player, 2, book)? != Some(1) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "enchanted book did not retain its stored enchantment",
        ));
    }
    player_inventory::remove_enchantment(player, 2, book)?;
    if player_inventory::read_enchantment_level(player, 2, book)?.is_some() {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "stored enchantment was not removed from the enchanted book",
        ));
    }

    Ok("player inventory enchantment read, set, validation, and removal succeeded".to_owned())
}
