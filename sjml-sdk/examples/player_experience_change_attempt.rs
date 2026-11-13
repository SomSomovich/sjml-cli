#![cfg_attr(
    not(all(target_arch = "wasm32", target_os = "unknown")),
    allow(dead_code)
)]

use sjml::{
    event,
    events::{self, EventPriority, PlayerExperienceChangeAction, PlayerExperienceChangeAttempt},
    mod_init, SjmlResult,
};

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("player.experience_change_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.experience_change_attempt")]
fn cap_small_level_xp_gain(
    attempt: PlayerExperienceChangeAttempt,
) -> SjmlResult<PlayerExperienceChangeAction> {
    // The manifest must grant events.subscribe, events.cancel, and player.vitals.write.
    if attempt.level_before < 30 && attempt.requested_amount > 5 {
        Ok(PlayerExperienceChangeAction::SetAmount(5))
    } else {
        Ok(PlayerExperienceChangeAction::Pass)
    }
}
