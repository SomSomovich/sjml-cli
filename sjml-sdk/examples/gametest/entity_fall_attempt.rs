use std::sync::atomic::{AtomicBool, Ordering};

use sjml::{
    event,
    events::{self, EntityFallAttempt, EventPriority},
    SjmlResult,
};

static ARMED: AtomicBool = AtomicBool::new(true);

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.fall_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.fall_attempt")]
fn veto_one_test_fall(event: EntityFallAttempt) -> SjmlResult<bool> {
    let matches_test_entity = event.entity_type_id == "minecraft:pig"
        && event.dimension_id == "minecraft:overworld"
        && canonical_uuid(&event.entity_id)
        && event.x.is_finite()
        && event.y.is_finite()
        && event.z.is_finite()
        && event.fall_distance == 7.5
        && event.damage_multiplier == 1.0;
    Ok(matches_test_entity && ARMED.swap(false, Ordering::Relaxed))
}

fn canonical_uuid(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 36
        && [8, 13, 18, 23]
            .into_iter()
            .all(|index| bytes[index] == b'-')
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| [8, 13, 18, 23].contains(&index) || byte.is_ascii_hexdigit())
}
