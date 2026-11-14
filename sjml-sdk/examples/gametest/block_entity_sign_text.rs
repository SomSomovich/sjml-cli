use sjml::prelude::*;

#[derive(StableDecode)]
#[sjml(version = 1)]
struct SignTextGameTestArguments {
    x: i32,
    y: i32,
    z: i32,
    waxed_x: i32,
    waxed_y: i32,
    waxed_z: i32,
}

#[command(name = "block_entity.sign_text.check")]
fn check_sign_text(arguments: SignTextGameTestArguments) -> SjmlResult<String> {
    let world = world::get("minecraft:overworld")?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the overworld must already be loaded",
        )
    })?;
    let sign = world
        .block_entity(BlockPos {
            x: arguments.x,
            y: arguments.y,
            z: arguments.z,
        })?
        .ok_or_else(|| SjmlError::new(ErrorCode::InvalidArgument, "the test sign is missing"))?;
    let waxed_sign = world
        .block_entity(BlockPos {
            x: arguments.waxed_x,
            y: arguments.waxed_y,
            z: arguments.waxed_z,
        })?
        .ok_or_else(|| {
            SjmlError::new(ErrorCode::InvalidArgument, "the waxed test sign is missing")
        })?;
    let original_front = sign.sign_text(SignSide::Front)?;
    let original_back = sign.sign_text(SignSide::Back)?;
    let front = SignTextLines::new([
        "guest front one".to_owned(),
        "guest front two".to_owned(),
        String::new(),
        String::new(),
    ])?;
    let back = SignTextLines::new([
        "guest back one".to_owned(),
        String::new(),
        "guest back three".to_owned(),
        String::new(),
    ])?;

    let check = (|| {
        sign.set_sign_text(SignSide::Front, &front)?;
        sign.set_sign_text(SignSide::Back, &back)?;
        if sign.sign_text(SignSide::Front)? != front || sign.sign_text(SignSide::Back)? != back {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "sign text did not round-trip on both faces",
            ));
        }
        let error = waxed_sign
            .set_sign_text(SignSide::Front, &front)
            .unwrap_err();
        if error.code() != ErrorCode::PermissionDenied {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "the sign text API did not report the waxed-sign write as denied",
            ));
        }
        Ok(())
    })();
    let restore_front = sign.set_sign_text(SignSide::Front, &original_front);
    let restore_back = sign.set_sign_text(SignSide::Back, &original_back);
    check?;
    restore_front?;
    restore_back?;
    Ok("both sign faces round-tripped and waxed signs were rejected".to_owned())
}
