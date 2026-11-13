use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use sjml::{
    command, event,
    events::{
        self, AdvancementProgressOperation, EventPriority, ExplosionAttempt,
        PlayerAdvancementEarned, PlayerAdvancementProgressEvent, PlayerSleepAttempt,
    },
    ErrorCode, SjmlError, SjmlResult, StableDecode,
};

const STORY_ROOT_ADVANCEMENT: &str = "minecraft:story/root";
static SAW_STORY_ROOT_EARNED: AtomicBool = AtomicBool::new(false);
static ADVANCEMENT_PROGRESS_STATES: AtomicU32 = AtomicU32::new(0);
static SAW_EXPLOSION_VETO: AtomicBool = AtomicBool::new(false);
static SAW_PLAYER_SLEEP_VETO: AtomicBool = AtomicBool::new(false);

/// Called by the GameTest guest's main module init export.
pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("player.advancement_earned", EventPriority::Normal)?;
    events::subscribe("player.advancement_progress", EventPriority::Normal)?;
    events::subscribe("world.explosion_attempt", EventPriority::Normal)?;
    events::subscribe("player.sleep_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.advancement_earned")]
fn observe_story_root(event: PlayerAdvancementEarned) -> SjmlResult<()> {
    if event.advancement_id == STORY_ROOT_ADVANCEMENT {
        SAW_STORY_ROOT_EARNED.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[event(name = "player.advancement_progress")]
fn observe_advancement_progress(event: PlayerAdvancementProgressEvent) -> SjmlResult<()> {
    if event.advancement_id == STORY_ROOT_ADVANCEMENT && event.criterion_id == "crafting_table" {
        match (event.operation, event.is_done) {
            (AdvancementProgressOperation::Grant, true) => {
                ADVANCEMENT_PROGRESS_STATES.fetch_or(1, Ordering::Relaxed);
            }
            (AdvancementProgressOperation::Revoke, false) => {
                ADVANCEMENT_PROGRESS_STATES.fetch_or(2, Ordering::Relaxed);
            }
            _ => {}
        }
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

#[event(name = "player.sleep_attempt")]
fn cancel_test_sleep(event: PlayerSleepAttempt) -> SjmlResult<bool> {
    let veto = event.can_veto && event.dimension_id == "minecraft:overworld";
    if veto {
        SAW_PLAYER_SLEEP_VETO.store(true, Ordering::Relaxed);
    }
    Ok(veto)
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct NoArguments {}

#[command(name = "gameplay.event.check")]
fn check_gameplay_event(_: NoArguments) -> SjmlResult<String> {
    let saw_story_root = SAW_STORY_ROOT_EARNED.swap(false, Ordering::Relaxed);
    let saw_sleep_veto = SAW_PLAYER_SLEEP_VETO.swap(false, Ordering::Relaxed);
    if saw_story_root || saw_sleep_veto {
        Ok("guest gameplay-event callback delivered".to_owned())
    } else {
        Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "no expected player advancement or sleep-attempt event was observed",
        ))
    }
}

#[command(name = "gameplay.advancement_progress.check")]
fn check_advancement_progress(_: NoArguments) -> SjmlResult<String> {
    if ADVANCEMENT_PROGRESS_STATES.swap(0, Ordering::Relaxed) == 3 {
        Ok(
            "guest observed criterion grant and revoke with post-change completion state"
                .to_owned(),
        )
    } else {
        Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "guest did not observe both criterion transitions with their post-change completion states",
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
