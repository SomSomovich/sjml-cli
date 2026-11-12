use sjml::{command, world, BlockPos, ErrorCode, SjmlError, SjmlResult, StableDecode};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct RedstoneSignalCheck {
    x: i32,
    y: i32,
    z: i32,
    expected: i32,
    expect_unloaded: bool,
}

#[command(name = "world.redstone.signal.check")]
fn check(arguments: RedstoneSignalCheck) -> SjmlResult<String> {
    let overworld = world::get("minecraft:overworld")?
        .ok_or_else(|| SjmlError::new(ErrorCode::InvalidArgument, "the Overworld is not loaded"))?;
    let signal = world::best_neighbor_signal(
        overworld,
        BlockPos {
            x: arguments.x,
            y: arguments.y,
            z: arguments.z,
        },
    )?;

    if arguments.expect_unloaded {
        return if signal.is_none() {
            Ok("redstone neighborhood stayed unloaded".to_owned())
        } else {
            Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "an unavailable redstone neighborhood unexpectedly returned a signal",
            ))
        };
    }

    if !(0..=15).contains(&arguments.expected) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "expected redstone signal must be in 0..=15",
        ));
    }
    if signal == Some(arguments.expected as u8) {
        Ok(format!("redstone signal is {}", arguments.expected))
    } else {
        Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            format!(
                "expected signal {}, received {signal:?}",
                arguments.expected
            ),
        ))
    }
}
