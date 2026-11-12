use std::sync::atomic::{AtomicBool, Ordering};

use sjml::{
    command, event,
    events::{self, EntityEffectApplicationAttempt, EventPriority},
    ErrorCode, SjmlError, SjmlResult, StableDecode,
};

static SAW_TEST_POISON_VETO: AtomicBool = AtomicBool::new(false);

/// Called by the GameTest guest's main module init export.
pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.effect_application", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.effect_application")]
fn veto_test_poison_application(event: EntityEffectApplicationAttempt) -> SjmlResult<bool> {
    let veto = event.entity_type_id == "minecraft:cow"
        && event.effect_id == "minecraft:poison"
        && event.duration_ticks == 1234
        && event.amplifier == 2
        && event.source_entity_id.is_some()
        && event.source_entity_type_id.as_deref() == Some("minecraft:pig");
    if veto {
        SAW_TEST_POISON_VETO.store(true, Ordering::Relaxed);
    }
    Ok(veto)
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct NoArguments {}

#[command(name = "entity.effect_application.check")]
fn check_poison_veto(_: NoArguments) -> SjmlResult<String> {
    if SAW_TEST_POISON_VETO.swap(false, Ordering::Relaxed) {
        Ok("guest observed and vetoed the sourced poison effect".to_owned())
    } else {
        Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity.effect_application callback did not veto the sourced poison effect",
        ))
    }
}
