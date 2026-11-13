use std::sync::atomic::{AtomicBool, Ordering};

use sjml::{
    event,
    events::{self, EntityMountAttempt, EventPriority},
    SjmlResult,
};

static MOUNT_ARMED: AtomicBool = AtomicBool::new(true);
static DISMOUNT_ARMED: AtomicBool = AtomicBool::new(true);

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.mount_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.mount_attempt")]
fn veto_one_pig_mount(event: EntityMountAttempt) -> SjmlResult<bool> {
    let targets_minecart = event
        .mounted_entity
        .as_ref()
        .is_some_and(|target| target.entity_type_id == "minecraft:minecart");
    let has_valid_identity =
        event.dimension_id == "minecraft:overworld" && canonical_uuid(&event.entity_id);
    let matches_mount_test = event.mounting
        && event.entity_type_id == "minecraft:pig"
        && targets_minecart
        && has_valid_identity;
    let matches_dismount_test =
        !event.mounting && event.entity_type_id == "minecraft:cow" && has_valid_identity;
    Ok(
        (matches_mount_test && MOUNT_ARMED.swap(false, Ordering::Relaxed))
            || (matches_dismount_test && DISMOUNT_ARMED.swap(false, Ordering::Relaxed)),
    )
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
