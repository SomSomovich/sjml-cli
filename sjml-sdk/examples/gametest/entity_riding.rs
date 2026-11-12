use sjml::{command, entity, world, BlockPos, ErrorCode, SjmlError, SjmlResult, StableDecode};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct RidingArguments {
    passenger_uuid: String,
    vehicle_uuid: String,
    other_vehicle_uuid: String,
    center_x: i32,
    center_y: i32,
    center_z: i32,
}

#[command(name = "entity.riding.check")]
fn exercise_entity_riding(arguments: RidingArguments) -> SjmlResult<String> {
    let overworld = world::get("minecraft:overworld")?
        .ok_or_else(|| failure("the GameTest Overworld is unavailable"))?;
    let nearby = entity::within(
        overworld,
        BlockPos {
            x: arguments.center_x,
            y: arguments.center_y,
            z: arguments.center_z,
        },
        8.0,
        8.0,
        Some("minecraft:pig"),
    )?;
    let mut passenger = None;
    let mut vehicle = None;
    let mut other_vehicle = None;
    for handle in nearby {
        match entity::snapshot(handle)?.uuid.as_str() {
            uuid if uuid == arguments.passenger_uuid => passenger = Some(handle),
            uuid if uuid == arguments.vehicle_uuid => vehicle = Some(handle),
            uuid if uuid == arguments.other_vehicle_uuid => other_vehicle = Some(handle),
            _ => {}
        }
    }
    let passenger =
        passenger.ok_or_else(|| failure("GameTest passenger is outside the local query"))?;
    let vehicle = vehicle.ok_or_else(|| failure("GameTest vehicle is outside the local query"))?;
    let other_vehicle = other_vehicle
        .ok_or_else(|| failure("other GameTest vehicle is outside the local query"))?;

    if entity::riding_snapshot(passenger)?.vehicle.is_some() {
        return Err(failure("GameTest passenger unexpectedly started mounted"));
    }
    if !entity::start_riding(passenger, vehicle)? {
        return Err(failure("Minecraft refused the ordinary GameTest mount"));
    }

    let passenger_state = entity::riding_snapshot(passenger)?;
    if passenger_state
        .vehicle
        .as_ref()
        .map(|entry| entry.uuid.as_str())
        != Some(arguments.vehicle_uuid.as_str())
    {
        return Err(failure(
            "vehicle snapshot did not report the mounted vehicle",
        ));
    }
    let vehicle_state = entity::riding_snapshot(vehicle)?;
    if !vehicle_state
        .passengers
        .iter()
        .any(|entry| entry.uuid == arguments.passenger_uuid)
    {
        return Err(failure("vehicle snapshot omitted its mounted passenger"));
    }

    if entity::start_riding(passenger, other_vehicle)? {
        return Err(failure(
            "a mounted passenger was silently moved to another vehicle",
        ));
    }
    if entity::riding_snapshot(passenger)?
        .vehicle
        .as_ref()
        .map(|entry| entry.uuid.as_str())
        != Some(arguments.vehicle_uuid.as_str())
    {
        return Err(failure("a rejected re-mount changed the existing vehicle"));
    }

    if !entity::stop_riding(passenger)? || entity::stop_riding(passenger)? {
        return Err(failure(
            "entity dismount did not report its state transition",
        ));
    }
    if entity::riding_snapshot(passenger)?.vehicle.is_some() {
        return Err(failure("dismounted passenger still reports a vehicle"));
    }

    if entity::start_riding(passenger, other_vehicle)? {
        return Err(failure(
            "vanilla's dismount cooldown must reject an immediate second mount",
        ));
    }
    if entity::riding_snapshot(passenger)?.vehicle.is_some() {
        return Err(failure(
            "rejected immediate re-mount changed the dismounted state",
        ));
    }

    Ok("entity mount snapshots and non-forced mount lifecycle passed".to_owned())
}

fn failure(message: &'static str) -> SjmlError {
    SjmlError::new(ErrorCode::InvalidArgument, message)
}
