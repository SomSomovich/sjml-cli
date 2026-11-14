#![cfg_attr(
    not(all(target_arch = "wasm32", target_os = "unknown")),
    allow(dead_code)
)]

use sjml::events::{PlayerStatisticChanged, PLAYER_STATISTIC_CHANGED_EVENT_NAME};
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe(PLAYER_STATISTIC_CHANGED_EVENT_NAME, EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.statistic_changed")]
fn statistic_changed(event: PlayerStatisticChanged) -> SjmlResult<()> {
    host::log(
        LogLevel::Info,
        &format!(
            "player {} changed {} / {}: {} -> {}",
            event.player_id,
            event.statistic_type_id,
            event.statistic_id,
            event.previous_value,
            event.current_value,
        ),
    )?;
    Ok(())
}
