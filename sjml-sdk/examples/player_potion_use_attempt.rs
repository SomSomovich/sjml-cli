#![cfg_attr(
    not(all(target_arch = "wasm32", target_os = "unknown")),
    allow(dead_code)
)]

use sjml::{
    event,
    events::{self, EventPriority, PlayerPotionUseAttempt, PlayerPotionUseAttemptAction},
    mod_init, SjmlResult,
};

#[mod_init]
fn initialize() -> SjmlResult<()> {
    // The module manifest must grant events.subscribe and events.cancel.
    events::subscribe("player.potion_use_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.potion_use_attempt")]
fn block_plain_potion(attempt: PlayerPotionUseAttempt) -> SjmlResult<PlayerPotionUseAttemptAction> {
    if attempt.item_id == "minecraft:potion" && attempt.hand == "main_hand" {
        Ok(PlayerPotionUseAttemptAction::Cancel)
    } else {
        Ok(PlayerPotionUseAttemptAction::Pass)
    }
}
