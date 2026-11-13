#![cfg_attr(
    not(all(target_arch = "wasm32", target_os = "unknown")),
    allow(dead_code)
)]

use sjml::{
    event,
    events::{
        self, EventPriority, PlayerExperienceLevelChangeAction, PlayerExperienceLevelChangeAttempt,
    },
    mod_init, SjmlResult,
};

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe(
        "player.experience_level_change_attempt",
        EventPriority::Normal,
    )?;
    Ok(())
}

#[event(name = "player.experience_level_change_attempt")]
fn handle_level_change(
    attempt: PlayerExperienceLevelChangeAttempt,
) -> SjmlResult<PlayerExperienceLevelChangeAction> {
    // The manifest must grant events.subscribe, events.cancel, and player.vitals.write.
    match attempt.requested_levels {
        5 => Ok(PlayerExperienceLevelChangeAction::SetLevels(2)),
        -3 => Ok(PlayerExperienceLevelChangeAction::Cancel),
        _ => Ok(PlayerExperienceLevelChangeAction::Pass),
    }
}
