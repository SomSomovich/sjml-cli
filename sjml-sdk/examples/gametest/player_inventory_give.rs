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
    player_inventory::set_item_custom_name(target, 34, "move-preserved emerald")?;
    player_inventory::set_lore(target, 34, &["move-preserved lore".to_owned()])?;
    player_inventory::set_item_custom_name(target, 35, "count-preserved name")?;
    player_inventory::set_lore(target, 35, &["count-preserved lore".to_owned()])?;
    player_inventory::set_count(target, 35, 60)?;
    let reduced = player_inventory::read(target)?;
    if reduced.slots[35]
        .as_ref()
        .is_none_or(|stack| stack.count != 60)
        || player_inventory::read_lore(target, 35)? != ["count-preserved lore"]
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player stack count did not update while preserving lore",
        ));
    }
    let too_many = player_inventory::set_count(target, 35, 65).unwrap_err();
    if too_many.code() != ErrorCode::InvalidArgument {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player stack count must obey item and slot limits",
        ));
    }
    let empty_slot = player_inventory::set_count(target, 40, 1).unwrap_err();
    if empty_slot.code() != ErrorCode::InvalidArgument {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player stack count must reject empty slots",
        ));
    }
    player_inventory::set_count(target, 35, 64)?;
    player_inventory::set_slot(target, 33, PlayerInventorySlotUpdate::Clear)?;
    player_inventory::move_stack(target, 34, 33, 2)?;
    if player_inventory::read_lore(target, 33)? != ["move-preserved lore"]
        || player_inventory::read_lore(target, 34)? != ["move-preserved lore"]
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player inventory move must preserve stack components on both slots",
        ));
    }
    let insufficient_source = player_inventory::move_stack(target, 34, 33, 2).unwrap_err();
    if insufficient_source.code() != ErrorCode::InvalidArgument {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player inventory move must reject a request larger than its source stack",
        ));
    }
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
            .is_none_or(|stack| stack.item_id != "minecraft:emerald" || stack.count != 1)
        || snapshot
            .slots
            .get(33)
            .and_then(Option::as_ref)
            .is_none_or(|stack| stack.item_id != "minecraft:emerald" || stack.count != 2)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player inventory grant, count change, or slot-to-slot move produced the wrong stacks",
        ));
    }
    Ok("player inventory item grant inserted counts and preserved the remainder".to_owned())
}
