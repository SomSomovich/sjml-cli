use sjml::{
    command, world,
    world::{ExplosionInteraction, RaycastPoint},
    BlockPos, ErrorCode, SjmlError, SjmlResult, StableDecode,
};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct ExplosionArguments {
    x: i32,
    y: i32,
    z: i32,
    power: i32,
    fire: bool,
    interaction: i32,
}

#[command(name = "world.explosion.create")]
fn create_explosion(arguments: ExplosionArguments) -> SjmlResult<String> {
    let world = world::get("minecraft:overworld")?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the overworld must already be loaded",
        )
    })?;
    let interaction = match arguments.interaction {
        0 => ExplosionInteraction::None,
        1 => ExplosionInteraction::Block,
        _ => {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "explosion interaction must be 0 (none) or 1 (block)",
            ));
        }
    };
    let power = arguments.power as f32;
    world::create_explosion(
        world,
        RaycastPoint::new(
            f64::from(arguments.x) + 0.5,
            f64::from(arguments.y) + 0.5,
            f64::from(arguments.z) + 0.5,
        ),
        power,
        arguments.fire,
        interaction,
    )?;
    Ok("guest explosion created".to_owned())
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct LightningArguments {
    x: i32,
    y: i32,
    z: i32,
    visual_only: bool,
}

#[command(name = "world.lightning.strike")]
fn strike_lightning(arguments: LightningArguments) -> SjmlResult<String> {
    let world = world::get("minecraft:overworld")?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the overworld must already be loaded",
        )
    })?;
    world::strike_lightning(
        world,
        BlockPos {
            x: arguments.x,
            y: arguments.y,
            z: arguments.z,
        },
        arguments.visual_only,
    )?;
    Ok("guest lightning strike created".to_owned())
}
