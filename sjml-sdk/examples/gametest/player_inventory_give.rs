use sjml::{
    command, player, player_inventory, ErrorCode, PlayerHandle, PlayerInventorySlotUpdate,
    SjmlError, SjmlResult, StableDecode,
};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct PlayerInventoryGiveArguments {
    player_id: String,
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

#[command(name = "player.inventory.give_item")]
fn give_player_inventory_item(arguments: PlayerInventoryGiveArguments) -> SjmlResult<String> {
    let target = find_player(&arguments.player_id)?;
    let merged = player_inventory::give_item(target, "minecraft:diamond", 8)?;
    if merged != 2 {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "grant must return the two items inserted into the existing diamond stack",
        ));
    }
    let rejected = player_inventory::give_item(target, "minecraft:emerald", 1)?;
    if rejected != 0 {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "a full main inventory must report that no item was inserted",
        ));
    }

    player_inventory::set_slot(target, 34, PlayerInventorySlotUpdate::Clear)?;
    let placed = player_inventory::give_item(target, "minecraft:emerald", 3)?;
    let snapshot = player_inventory::read(target)?;
    if placed != 3
        || snapshot
            .slots
            .get(35)
            .and_then(Option::as_ref)
            .is_none_or(|stack| stack.item_id != "minecraft:diamond" || stack.count != 64)
        || snapshot
            .slots
            .get(34)
            .and_then(Option::as_ref)
            .is_none_or(|stack| stack.item_id != "minecraft:emerald" || stack.count != 3)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player item grant did not merge, report capacity, or fill the new empty slot correctly",
        ));
    }
    Ok("player inventory item grant inserted counts and preserved the remainder".to_owned())
}
