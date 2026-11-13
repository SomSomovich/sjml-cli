use sjml::{
    event,
    events::{self, EventPriority, PlayerTotemUseAttempt, PlayerTotemUseAttemptAction},
    ErrorCode, SjmlError, SjmlResult,
};
use std::sync::Mutex;

static LAST_TOTEM_USE_ATTEMPT: Mutex<Option<PlayerTotemUseAttempt>> = Mutex::new(None);

pub(super) fn initialize() -> SjmlResult<()> {
    *LAST_TOTEM_USE_ATTEMPT.lock().map_err(|_| {
        SjmlError::new(
            ErrorCode::UnknownErrorCode,
            "player.totem_use_attempt state lock is poisoned",
        )
    })? = None;
    events::subscribe("player.totem_use_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.totem_use_attempt")]
fn cancel_generic_damage_totem(
    attempt: PlayerTotemUseAttempt,
) -> SjmlResult<PlayerTotemUseAttemptAction> {
    let should_cancel = attempt.totem_id == "minecraft:totem_of_undying"
        && attempt.damage_type_id == "minecraft:generic"
        && attempt.count == 1
        && attempt.hand == "off_hand";
    *LAST_TOTEM_USE_ATTEMPT.lock().map_err(|_| {
        SjmlError::new(
            ErrorCode::UnknownErrorCode,
            "player.totem_use_attempt state lock is poisoned",
        )
    })? = Some(attempt);
    Ok(if should_cancel {
        PlayerTotemUseAttemptAction::Cancel
    } else {
        PlayerTotemUseAttemptAction::Pass
    })
}

/// Checks the detached totem callback recorded by the existing item-use GameTest command.
pub(super) fn matches_existing_item_use_check(
    player_id: &str,
    dimension_id: &str,
    hand: &str,
    totem_id: &str,
    count: i32,
) -> SjmlResult<bool> {
    let observed = LAST_TOTEM_USE_ATTEMPT.lock().map_err(|_| {
        SjmlError::new(
            ErrorCode::UnknownErrorCode,
            "player.totem_use_attempt state lock is poisoned",
        )
    })?;
    Ok(observed.as_ref().is_some_and(|attempt| {
        attempt.player_id == player_id
            && attempt.dimension_id == dimension_id
            && attempt.hand == hand
            && attempt.totem_id == totem_id
            && attempt.count == u32::try_from(count).unwrap_or(u32::MAX)
            && attempt.damage_type_id == "minecraft:generic"
            && attempt.source_entity_id.is_none()
            && attempt.source_entity_type_id.is_none()
    }))
}
