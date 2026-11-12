use sjml::{command, world, ErrorCode, SjmlError, SjmlResult, StableDecode, TagRegistry};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct TagMembersArguments {
    query: String,
}

#[command(name = "world.tag_members.check")]
fn check_current_registry_tags(arguments: TagMembersArguments) -> SjmlResult<String> {
    let world = world::get("minecraft:overworld")?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::UnknownErrorCode,
            "the GameTest Overworld is unavailable",
        )
    })?;
    let Some((registry, tag_id)) = arguments.query.split_once('|') else {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "tag query must contain one registry selector and a tag ID",
        ));
    };
    let result = match registry {
        "block" => world::block_tag_members(world, tag_id)?,
        "item" => world::item_tag_members(world, tag_id)?,
        "fluid" => world::fluid_tag_members(world, tag_id)?,
        "entity_type" => world::entity_type_tag_members(world, tag_id)?,
        "missing" => world::tag_members(world, TagRegistry::Block, tag_id)?,
        _ => {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "unknown registry selector for tag-members test",
            ));
        }
    };
    let expected = match registry {
        "block" => Some("minecraft:oak_log"),
        "item" => Some("minecraft:oak_planks"),
        "fluid" => Some("minecraft:water"),
        "entity_type" => Some("minecraft:skeleton"),
        "missing" => None,
        _ => {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "unknown registry selector for tag-members test",
            ));
        }
    };
    let valid = match expected {
        Some(expected) => result
            .as_ref()
            .is_some_and(|members| members.iter().any(|id| id == expected)),
        None => result.is_none(),
    };
    if !valid {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "registry tag enumeration returned an unexpected set of members",
        ));
    }
    Ok(format!(
        "{registry} tag enumeration matched the expected registry contents"
    ))
}
