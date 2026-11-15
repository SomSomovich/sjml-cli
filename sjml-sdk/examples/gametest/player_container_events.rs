use sjml::{
    event,
    events::{self, EventPriority},
    player_container_events::{PlayerContainerClosed, PlayerContainerOpened},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("player.container_open", EventPriority::Normal)?;
    events::subscribe("player.container_close", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.container_open")]
fn observe_container_open(event: PlayerContainerOpened) -> SjmlResult<()> {
    let _tracked =
        event.menu_id == "minecraft:generic_9x1" && event.container_id >= 0 && event.slot_count > 0;
    Ok(())
}

#[event(name = "player.container_close")]
fn observe_container_close(event: PlayerContainerClosed) -> SjmlResult<()> {
    let _tracked =
        event.menu_id == "minecraft:generic_9x1" && event.container_id >= 0 && event.slot_count > 0;
    Ok(())
}
