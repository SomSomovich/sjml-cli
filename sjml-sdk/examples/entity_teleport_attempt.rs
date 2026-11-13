//! Synchronous entity teleport policy callback.

#![cfg_attr(
    not(all(target_arch = "wasm32", target_os = "unknown")),
    allow(dead_code)
)]

use sjml::{
    event,
    events::{self, EventPriority},
    mod_init, EntityTeleportAttempt, EntityTeleportAttemptAction, SjmlResult,
};

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("entity.teleport_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "entity.teleport_attempt")]
fn guard_long_teleports(event: EntityTeleportAttempt) -> SjmlResult<EntityTeleportAttemptAction> {
    let distance_squared = (event.target_x - event.from_x).powi(2)
        + (event.target_y - event.from_y).powi(2)
        + (event.target_z - event.from_z).powi(2);
    if distance_squared > 128.0_f64.powi(2) {
        Ok(EntityTeleportAttemptAction::Cancel)
    } else {
        Ok(EntityTeleportAttemptAction::Pass)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
