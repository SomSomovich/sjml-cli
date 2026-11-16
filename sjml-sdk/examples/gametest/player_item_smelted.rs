use sjml::player_item_smelted::{PlayerItemSmelted, EVENT_NAME};
use sjml::{
    event,
    events::{self, EventPriority},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe(EVENT_NAME, EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.item_smelted")]
fn observe_player_item_smelted(event: PlayerItemSmelted) -> SjmlResult<()> {
    let _ = (
        event.player_id,
        event.dimension_id,
        event.item_id,
        event.count,
    );
    Ok(())
}
