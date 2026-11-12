use sjml::{command, world, BlockPos, ErrorCode, SjmlError, SjmlResult, StableDecode};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct FluidStateCheck {
    x: i32,
    y: i32,
    z: i32,
}

#[command(name = "world.fluid.at.check")]
fn check_loaded_fluid_states(arguments: FluidStateCheck) -> SjmlResult<String> {
    let overworld = world::get("minecraft:overworld")?
        .ok_or_else(|| SjmlError::new(ErrorCode::InvalidArgument, "the Overworld is not loaded"))?;
    let source_position = BlockPos {
        x: arguments.x,
        y: arguments.y,
        z: arguments.z,
    };
    let dry_position = BlockPos {
        x: arguments.x + 1,
        y: arguments.y,
        z: arguments.z,
    };
    let source = world::fluid_at(overworld, source_position)?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the loaded source-water position was unavailable",
        )
    })?;
    let dry = world::fluid_at(overworld, dry_position)?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the loaded dry position was unavailable",
        )
    })?;

    if source.fluid_id != "minecraft:water"
        || source.amount != 8
        || !source.source
        || source.height.to_bits() != (8.0_f32 / 9.0_f32).to_bits()
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "fluid_at did not return the expected source-water snapshot",
        ));
    }
    if dry.fluid_id != "minecraft:empty" || dry.amount != 0 || dry.source || dry.height != 0.0 {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "fluid_at did not return the expected empty-fluid snapshot for a dry block",
        ));
    }
    Ok("fluid_at returned source water and the empty fluid state".to_owned())
}
