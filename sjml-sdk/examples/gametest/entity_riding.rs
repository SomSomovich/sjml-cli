use sjml::{command, entity, EntityHandle, ErrorCode, SjmlError, SjmlResult, StableDecode};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct RidingArguments {
    passenger_uuid: String,
    vehicle_uuid: String,
    other_vehicle_uuid: String,
}

#[command(name = "entity.riding.check")]
fn exercise_entity_riding(arguments: RidingArguments) -> SjmlResult<String> {
    let passenger = find_entity(&arguments.passenger_uuid)?;
    let vehicle = find_entity(&arguments.vehicle_uuid)?;
    let other_vehicle = find_entity(&arguments.other_vehicle_uuid)?;

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

fn find_entity(uuid: &str) -> SjmlResult<EntityHandle> {
    for handle in entity::loaded()? {
        if entity::snapshot(handle)?.uuid == uuid {
            return Ok(handle);
        }
    }
    Err(failure(
        "requested GameTest entity is not present in loaded levels",
    ))
}

fn failure(message: &'static str) -> SjmlError {
    SjmlError::new(ErrorCode::InvalidArgument, message)
}
