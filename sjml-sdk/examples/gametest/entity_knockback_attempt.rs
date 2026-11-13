use sjml::prelude::{EntityKnockbackAttempt, EventPriority};
use sjml::{command, event, events, ErrorCode, SjmlError, SjmlResult};
use std::sync::atomic::{AtomicBool, Ordering};

static SAW_KNOCKBACK_ATTEMPT: AtomicBool = AtomicBool::new(false);
static ARM_KNOCKBACK_ATTEMPT: AtomicBool = AtomicBool::new(false);

pub fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.knockback_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.knockback_attempt")]
fn cancel_test_pig_knockback(event: EntityKnockbackAttempt) -> SjmlResult<bool> {
    let cancel = event.entity_type_id == "minecraft:pig"
        && event.original_strength > 0.0
        && event.strength > 0.0
        && event.original_ratio_x.is_finite()
        && event.original_ratio_z.is_finite()
        && event.ratio_x.is_finite()
        && event.ratio_z.is_finite()
        && ARM_KNOCKBACK_ATTEMPT.swap(false, Ordering::Relaxed);
    if cancel {
        SAW_KNOCKBACK_ATTEMPT.store(true, Ordering::Relaxed);
    }
    Ok(cancel)
}

#[command(name = "entity.knockback_attempt.arm")]
fn arm_knockback_cancellation(_: super::EntityDeathCheckArguments) -> SjmlResult<String> {
    SAW_KNOCKBACK_ATTEMPT.store(false, Ordering::Relaxed);
    ARM_KNOCKBACK_ATTEMPT.store(true, Ordering::Relaxed);
    Ok("armed pig knockback veto".to_owned())
}

#[command(name = "entity.knockback_attempt.check")]
fn check_knockback_cancellation(_: super::EntityDeathCheckArguments) -> SjmlResult<String> {
    if SAW_KNOCKBACK_ATTEMPT.swap(false, Ordering::Relaxed) {
        Ok("guest vetoed knockback for a pig".to_owned())
    } else {
        Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity.knockback_attempt callback did not observe and veto the pig",
        ))
    }
}
