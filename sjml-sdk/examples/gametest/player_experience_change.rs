use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use sjml::{
    event,
    events::{
        self, EventPriority, PlayerExperienceChangeAction, PlayerExperienceChangeAttempt,
        PlayerExperienceLevelChangeAction, PlayerExperienceLevelChangeAttempt,
    },
    ErrorCode, SjmlError, SjmlResult,
};

static AMOUNT_REPLACEMENTS: AtomicU32 = AtomicU32::new(0);
static CANCELLATIONS: AtomicU32 = AtomicU32::new(0);
static SAW_VALID_REPLACEMENT: AtomicBool = AtomicBool::new(false);
static SAW_VALID_CANCELLATION: AtomicBool = AtomicBool::new(false);
static LEVEL_REPLACEMENTS: AtomicU32 = AtomicU32::new(0);
static LEVEL_CANCELLATIONS: AtomicU32 = AtomicU32::new(0);
static SAW_VALID_LEVEL_REPLACEMENT: AtomicBool = AtomicBool::new(false);
static SAW_VALID_LEVEL_CANCELLATION: AtomicBool = AtomicBool::new(false);

pub(super) fn initialize() -> SjmlResult<()> {
    AMOUNT_REPLACEMENTS.store(0, Ordering::Relaxed);
    CANCELLATIONS.store(0, Ordering::Relaxed);
    SAW_VALID_REPLACEMENT.store(false, Ordering::Relaxed);
    SAW_VALID_CANCELLATION.store(false, Ordering::Relaxed);
    events::subscribe("player.experience_change_attempt", EventPriority::Normal)?;
    LEVEL_REPLACEMENTS.store(0, Ordering::Relaxed);
    LEVEL_CANCELLATIONS.store(0, Ordering::Relaxed);
    SAW_VALID_LEVEL_REPLACEMENT.store(false, Ordering::Relaxed);
    SAW_VALID_LEVEL_CANCELLATION.store(false, Ordering::Relaxed);
    events::subscribe(
        "player.experience_level_change_attempt",
        EventPriority::Normal,
    )?;
    Ok(())
}

#[event(name = "player.experience_level_change_attempt")]
fn replace_and_cancel_test_level_changes(
    event: PlayerExperienceLevelChangeAttempt,
) -> SjmlResult<PlayerExperienceLevelChangeAction> {
    if event.player_id.is_empty() || !(0..=20_000).contains(&event.level_before) {
        return Ok(PlayerExperienceLevelChangeAction::Pass);
    }

    match event.requested_levels {
        5 => {
            LEVEL_REPLACEMENTS.fetch_add(1, Ordering::Relaxed);
            SAW_VALID_LEVEL_REPLACEMENT.store(event.level_before == 0, Ordering::Relaxed);
            Ok(PlayerExperienceLevelChangeAction::SetLevels(2))
        }
        -3 => {
            LEVEL_CANCELLATIONS.fetch_add(1, Ordering::Relaxed);
            SAW_VALID_LEVEL_CANCELLATION.store(event.level_before == 2, Ordering::Relaxed);
            Ok(PlayerExperienceLevelChangeAction::Cancel)
        }
        _ => Ok(PlayerExperienceLevelChangeAction::Pass),
    }
}

#[event(name = "player.experience_change_attempt")]
fn replace_and_cancel_test_xp_changes(
    event: PlayerExperienceChangeAttempt,
) -> SjmlResult<PlayerExperienceChangeAction> {
    let valid_identity = !event.player_id.is_empty()
        && (0..=20_000).contains(&event.level_before)
        && event.progress_before.is_finite()
        && (0.0..=1.0).contains(&event.progress_before);
    if !valid_identity {
        return Ok(PlayerExperienceChangeAction::Pass);
    }

    match event.requested_amount {
        5 => {
            AMOUNT_REPLACEMENTS.fetch_add(1, Ordering::Relaxed);
            SAW_VALID_REPLACEMENT.store(
                event.level_before == 0 && event.progress_before == 0.0,
                Ordering::Relaxed,
            );
            Ok(PlayerExperienceChangeAction::SetAmount(2))
        }
        -3 => {
            CANCELLATIONS.fetch_add(1, Ordering::Relaxed);
            SAW_VALID_CANCELLATION.store(event.level_before == 0, Ordering::Relaxed);
            Ok(PlayerExperienceChangeAction::Cancel)
        }
        _ => Ok(PlayerExperienceChangeAction::Pass),
    }
}

pub(super) fn verify() -> SjmlResult<()> {
    let replacements = AMOUNT_REPLACEMENTS.swap(0, Ordering::Relaxed);
    let cancellations = CANCELLATIONS.swap(0, Ordering::Relaxed);
    let level_replacements = LEVEL_REPLACEMENTS.swap(0, Ordering::Relaxed);
    let level_cancellations = LEVEL_CANCELLATIONS.swap(0, Ordering::Relaxed);
    if replacements == 1
        && cancellations == 1
        && level_replacements == 1
        && level_cancellations == 1
        && SAW_VALID_REPLACEMENT.swap(false, Ordering::Relaxed)
        && SAW_VALID_CANCELLATION.swap(false, Ordering::Relaxed)
        && SAW_VALID_LEVEL_REPLACEMENT.swap(false, Ordering::Relaxed)
        && SAW_VALID_LEVEL_CANCELLATION.swap(false, Ordering::Relaxed)
    {
        Ok(())
    } else {
        Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player XP events must replace and cancel exactly one points delta and one levels delta",
        ))
    }
}
