use sjml::{command, world, ErrorCode, SjmlError, SjmlResult, StableDecode, WorldSpawnPoint};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct SpawnPointArguments {
    x: i32,
    y: i32,
    z: i32,
    angle: String,
}

impl SpawnPointArguments {
    fn angle(&self) -> SjmlResult<f32> {
        self.angle.parse::<f32>().map_err(|_| {
            SjmlError::new(
                ErrorCode::InvalidArgument,
                "spawn-point angle must be a finite number",
            )
        })
    }
}

fn overworld_handle() -> SjmlResult<sjml::WorldHandle> {
    world::get("minecraft:overworld")?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the loaded minecraft:overworld handle is required",
        )
    })
}

#[command(name = "world.spawn_point.control.set")]
fn set_spawn_point(arguments: SpawnPointArguments) -> SjmlResult<String> {
    let point = WorldSpawnPoint::new(arguments.x, arguments.y, arguments.z, arguments.angle()?)?;
    world::set_spawn_point(overworld_handle()?, point)?;
    Ok("guest updated the vanilla Overworld spawn point".to_owned())
}

#[command(name = "world.spawn_point.control.check")]
fn check_spawn_point(expected: SpawnPointArguments) -> SjmlResult<String> {
    let actual = world::spawn_point(overworld_handle()?)?;
    let expected_angle = expected.angle()?;
    if actual.x == expected.x
        && actual.y == expected.y
        && actual.z == expected.z
        && actual.angle.to_bits() == expected_angle.to_bits()
    {
        Ok("guest read the configured Overworld spawn point".to_owned())
    } else {
        Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "guest spawn-point snapshot did not match the expected coordinates and yaw",
        ))
    }
}
