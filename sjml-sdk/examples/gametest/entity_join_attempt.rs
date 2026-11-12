use std::sync::atomic::{AtomicBool, Ordering};

use sjml::{
    command, event,
    events::{self, EntityJoinAttempt, EventPriority},
    ErrorCode, SjmlError, SjmlResult, StableDecode,
};

static SAW_AREA_EFFECT_CLOUD_VETO: AtomicBool = AtomicBool::new(false);

/// Called by the GameTest guest's main module init export.
pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.join_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.join_attempt")]
fn veto_test_area_effect_cloud(event: EntityJoinAttempt) -> SjmlResult<bool> {
    let veto = event.entity_type_id == "minecraft:area_effect_cloud"
        && !event.loaded_from_disk
        && event.entity_id.len() == 36
        && !event.dimension_id.is_empty()
        && event.x.is_finite()
        && event.y.is_finite()
        && event.z.is_finite();
    if veto {
        SAW_AREA_EFFECT_CLOUD_VETO.store(true, Ordering::Relaxed);
    }
    Ok(veto)
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct NoArguments {}

#[command(name = "entity.join_attempt.check")]
fn check_area_effect_cloud_veto(_: NoArguments) -> SjmlResult<String> {
    if SAW_AREA_EFFECT_CLOUD_VETO.swap(false, Ordering::Relaxed) {
        Ok("guest observed and cancelled the area-effect-cloud join".to_owned())
    } else {
        Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity.join_attempt callback did not veto the test area effect cloud",
        ))
    }
}
