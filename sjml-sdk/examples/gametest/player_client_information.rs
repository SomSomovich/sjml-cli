use sjml::player_client_information::{PlayerClientInformation, EVENT_NAME};
use sjml::{
    event,
    events::{self, EventPriority},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe(EVENT_NAME, EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.client_information")]
fn observe_player_client_information(event: PlayerClientInformation) -> SjmlResult<()> {
    let _ = (
        event.player_id,
        event.dimension_id,
        event.language,
        event.view_distance,
        event.chat_visibility,
        event.chat_colors,
        event.model_customisation,
        event.main_hand,
        event.text_filtering_enabled,
        event.allows_listing,
    );
    Ok(())
}
