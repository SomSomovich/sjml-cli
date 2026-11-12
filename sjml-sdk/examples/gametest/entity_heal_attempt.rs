use std::sync::atomic::{AtomicBool, Ordering};

use sjml::{
    command, event,
    events::{self, EntityHealAttempt, EventPriority},
    ErrorCode, SjmlError, SjmlResult, StableDecode,
};

static SAW_COW_HEAL_VETO: AtomicBool = AtomicBool::new(false);

/// Called by the GameTest guest's main module init export.
pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.heal_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.heal_attempt")]
fn veto_test_cow_heal(event: EntityHealAttempt) -> SjmlResult<bool> {
    let veto = event.entity_type_id == "minecraft:cow" && event.amount == 123.0;
    if veto {
        SAW_COW_HEAL_VETO.store(true, Ordering::Relaxed);
    }
    Ok(veto)
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct NoArguments {}

#[command(name = "entity.heal.check")]
fn check_cow_heal_veto(_: NoArguments) -> SjmlResult<String> {
    if SAW_COW_HEAL_VETO.swap(false, Ordering::Relaxed) {
        Ok("guest observed and vetoed the cow heal".to_owned())
    } else {
        Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity.heal_attempt callback did not veto the GameTest cow heal",
        ))
    }
}
