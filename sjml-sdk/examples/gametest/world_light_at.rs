use sjml::{command, world, BlockPos, ErrorCode, SjmlError, SjmlResult, StableDecode};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct LightAtCheck {
    x: i32,
    y: i32,
    z: i32,
    expected_sky: i32,
    expected_block: i32,
    expect_empty: bool,
}

#[command(name = "world.light_at.check")]
fn check_loaded_light_layers(arguments: LightAtCheck) -> SjmlResult<String> {
    let overworld = world::get("minecraft:overworld")?
        .ok_or_else(|| SjmlError::new(ErrorCode::InvalidArgument, "the Overworld is not loaded"))?;
    let light = world::light_at(
        overworld,
        BlockPos {
            x: arguments.x,
            y: arguments.y,
            z: arguments.z,
        },
    )?;

    if arguments.expect_empty {
        return if light.is_none() {
            Ok("light_at returned None for an unavailable position".to_owned())
        } else {
            Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "an unavailable light position unexpectedly returned values",
            ))
        };
    }

    if !(0..=15).contains(&arguments.expected_sky) || !(0..=15).contains(&arguments.expected_block)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "expected sky and block light levels must each be in 0..=15",
        ));
    }
    let light = light.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "light_at returned None for an already-loaded valid position",
        )
    })?;
    if light.sky != arguments.expected_sky as u8 || light.block != arguments.expected_block as u8 {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            format!(
                "expected sky/block {}/{}, received {}/{}",
                arguments.expected_sky, arguments.expected_block, light.sky, light.block
            ),
        ));
    }
    Ok(format!(
        "light_at returned sky {} and block {}",
        light.sky, light.block
    ))
}
