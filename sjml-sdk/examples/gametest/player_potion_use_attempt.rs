use sjml::{
    event,
    events::{self, EventPriority, PlayerPotionUseAttempt, PlayerPotionUseAttemptAction},
    ErrorCode, SjmlError, SjmlResult,
};
use std::sync::Mutex;

static LAST_POTION_USE_ATTEMPT: Mutex<Option<PlayerPotionUseAttempt>> = Mutex::new(None);

pub(super) fn initialize() -> SjmlResult<()> {
    *LAST_POTION_USE_ATTEMPT.lock().map_err(|_| {
        SjmlError::new(
            ErrorCode::UnknownErrorCode,
            "player.potion_use_attempt state lock is poisoned",
        )
    })? = None;
    events::subscribe("player.potion_use_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.potion_use_attempt")]
fn cancel_test_potion(attempt: PlayerPotionUseAttempt) -> SjmlResult<PlayerPotionUseAttemptAction> {
    let should_cancel = attempt.item_id == "minecraft:potion"
        && attempt.count == 1
        && attempt.duration_ticks == 32
        && attempt.hand == "off_hand";
    *LAST_POTION_USE_ATTEMPT.lock().map_err(|_| {
        SjmlError::new(
            ErrorCode::UnknownErrorCode,
            "player.potion_use_attempt state lock is poisoned",
        )
    })? = Some(attempt);
    Ok(if should_cancel {
        PlayerPotionUseAttemptAction::Cancel
    } else {
        PlayerPotionUseAttemptAction::Pass
    })
}

/// Checks the potion callback snapshot from the existing player item-use GameTest command.
pub(super) fn matches_existing_item_use_check(
    player_id: &str,
    dimension_id: &str,
    hand: &str,
    item_id: &str,
    count: i32,
) -> SjmlResult<bool> {
    let observed = LAST_POTION_USE_ATTEMPT.lock().map_err(|_| {
        SjmlError::new(
            ErrorCode::UnknownErrorCode,
            "player.potion_use_attempt state lock is poisoned",
        )
    })?;
    Ok(observed.as_ref().is_some_and(|attempt| {
        attempt.player_id == player_id
            && attempt.dimension_id == dimension_id
            && attempt.hand == hand
            && attempt.item_id == item_id
            && attempt.count == u32::try_from(count).unwrap_or(u32::MAX)
            && attempt.duration_ticks == 32
    }))
}
