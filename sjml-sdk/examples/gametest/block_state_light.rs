use sjml::{command, world, BlockPos, ErrorCode, SjmlError, SjmlResult, StableDecode};

const BLOCK_ID: &str = "sjml_game_test:content/gametest_stateful_light_block";

#[derive(StableDecode)]
#[sjml(version = 1)]
struct BlockStateLightArguments {
    x: i32,
    y: i32,
    z: i32,
}

#[command(name = "block.state_light.toggle")]
fn toggle_registered_block_light_state(arguments: BlockStateLightArguments) -> SjmlResult<String> {
    let overworld = world::get("minecraft:overworld")?
        .ok_or_else(|| SjmlError::new(ErrorCode::InvalidArgument, "the Overworld is not loaded"))?;
    let position = BlockPos {
        x: arguments.x,
        y: arguments.y,
        z: arguments.z,
    };
    let state = world::block_states(overworld, &[position])?
        .into_iter()
        .next()
        .flatten()
        .filter(|state| state.block_id == BLOCK_ID)
        .ok_or_else(|| {
            SjmlError::new(
                ErrorCode::InvalidArgument,
                "the registered light-state block is not at the loaded position",
            )
        })?;
    let properties = world::block_properties(overworld, position)?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the registered light-state block properties are unavailable",
        )
    })?;
    let powered = properties
        .iter()
        .find(|property| property.name == "powered")
        .map(|property| property.value.as_str())
        .ok_or_else(|| {
            SjmlError::new(
                ErrorCode::InvalidArgument,
                "the registered light-state block has no powered property",
            )
        })?;
    let next_value = if powered == "false" {
        "true"
    } else if powered == "true" {
        "false"
    } else {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "the powered property is not a Boolean value",
        ));
    };
    world::set_block_properties(
        overworld,
        position,
        &state.block_id,
        &[world::BlockProperty {
            name: "powered".to_owned(),
            value: next_value.to_owned(),
        }],
    )?;
    let updated = world::block_properties(overworld, position)?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the registered light-state block disappeared after the update",
        )
    })?;
    if !updated
        .iter()
        .any(|property| property.name == "powered" && property.value == next_value)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "the registered block did not switch to the requested light state",
        ));
    }
    Ok(format!("powered={next_value}"))
}
