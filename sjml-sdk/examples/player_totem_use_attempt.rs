#![cfg_attr(
    not(all(target_arch = "wasm32", target_os = "unknown")),
    allow(dead_code)
)]

use sjml::{
    event,
    events::{self, EventPriority, PlayerTotemUseAttempt, PlayerTotemUseAttemptAction},
    mod_init, SjmlResult,
};

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("player.totem_use_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.totem_use_attempt")]
fn reject_void_damage_totems(
    event: PlayerTotemUseAttempt,
) -> SjmlResult<PlayerTotemUseAttemptAction> {
    if event.damage_type_id == "minecraft:out_of_world" {
        Ok(PlayerTotemUseAttemptAction::Cancel)
    } else {
        Ok(PlayerTotemUseAttemptAction::Pass)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
