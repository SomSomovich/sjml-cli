use sjml::{command, world, SjmlError, SjmlResult, StableDecode};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct ItemTagArguments {
    query: String,
}

#[command(name = "world.item_tag.check")]
fn check_loaded_item_tags(arguments: ItemTagArguments) -> SjmlResult<String> {
    let world = world::get("minecraft:overworld")?.ok_or_else(|| {
        SjmlError::new(
            sjml::ErrorCode::UnknownErrorCode,
            "the GameTest Overworld is unavailable",
        )
    })?;
    let (item_id, tag_id) = arguments.query.split_once('|').ok_or_else(|| {
        SjmlError::new(
            sjml::ErrorCode::InvalidArgument,
            "item-tag query must be item_id|tag_id",
        )
    })?;
    if tag_id.contains('|') {
        return Err(SjmlError::new(
            sjml::ErrorCode::InvalidArgument,
            "item-tag query contains more than one separator",
        ));
    }
    let member = world::item_has_tag(world, item_id, tag_id)?;
    let non_member = world::item_has_tag(world, "minecraft:stone", "minecraft:planks")?;
    let absent_tag = world::item_has_tag(world, "minecraft:oak_planks", "sjml_game_test:missing")?;
    if !member || non_member || absent_tag {
        return Err(SjmlError::new(
            sjml::ErrorCode::InvalidArgument,
            "item-tag lookup returned incorrect membership for a member, non-member, or absent tag",
        ));
    }
    Ok("current item tags matched oak planks and rejected stone and an absent tag".to_owned())
}
