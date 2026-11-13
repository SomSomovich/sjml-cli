#![cfg_attr(
    not(all(target_arch = "wasm32", target_os = "unknown")),
    allow(dead_code)
)]

use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("player.equipment_changed", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.equipment_changed")]
fn equipment_changed(event: PlayerEquipmentChanged) -> SjmlResult<()> {
    let previous = describe_stack(event.previous.as_ref());
    let current = describe_stack(event.current.as_ref());
    host::log(
        LogLevel::Info,
        &format!(
            "player {} changed {:?}: {} -> {}",
            event.player_id, event.slot, previous, current
        ),
    )?;
    Ok(())
}

fn describe_stack(stack: Option<&PlayerEquipmentItemSnapshot>) -> String {
    match stack {
        Some(stack) => format!(
            "{} x{} ({}/{})",
            stack.item_id, stack.count, stack.damage, stack.max_damage
        ),
        None => "empty".to_owned(),
    }
}
