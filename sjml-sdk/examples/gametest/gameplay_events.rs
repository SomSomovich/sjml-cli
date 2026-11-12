use std::sync::atomic::{AtomicBool, Ordering};

use sjml::{
    command, event,
    events::{self, EventPriority, ExplosionAttempt, PlayerAdvancementEarned},
    ErrorCode, SjmlError, SjmlResult, StableDecode,
};

const STORY_ROOT_ADVANCEMENT: &str = "minecraft:story/root";
static SAW_STORY_ROOT_EARNED: AtomicBool = AtomicBool::new(false);
static SAW_EXPLOSION_VETO: AtomicBool = AtomicBool::new(false);

/// Called by the GameTest guest's main module init export.
pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("player.advancement_earned", EventPriority::Normal)?;
    events::subscribe("world.explosion_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.advancement_earned")]
fn observe_story_root(event: PlayerAdvancementEarned) -> SjmlResult<()> {
    if event.advancement_id == STORY_ROOT_ADVANCEMENT {
        SAW_STORY_ROOT_EARNED.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[event(name = "world.explosion_attempt")]
fn cancel_test_explosion(event: ExplosionAttempt) -> SjmlResult<bool> {
    let veto = event.block_interaction != "keep" && event.radius >= 2.0;
    if veto {
        SAW_EXPLOSION_VETO.store(true, Ordering::Relaxed);
    }
    Ok(veto)
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct NoArguments {}

#[command(name = "player.advancement.check")]
fn check_story_root_earned(_: NoArguments) -> SjmlResult<String> {
    if SAW_STORY_ROOT_EARNED.swap(false, Ordering::Relaxed) {
        Ok("player advancement-earned callback delivered".to_owned())
    } else {
        Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player.advancement_earned callback did not observe minecraft:story/root",
        ))
    }
}

#[command(name = "world.explosion.check")]
fn check_explosion_veto(_: NoArguments) -> SjmlResult<String> {
    if SAW_EXPLOSION_VETO.swap(false, Ordering::Relaxed) {
        Ok("guest vetoed the GameTest explosion before world changes".to_owned())
    } else {
        Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "world.explosion_attempt callback did not veto the GameTest explosion",
        ))
    }
}
