use sjml::{
    event,
    events::{self, EventPriority, PlayerChatAttempt},
    SjmlResult,
};

const VETO_TEXT: &str = "sjml-gametest-chat-veto";

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("player.chat_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.chat_attempt")]
fn veto_marked_chat(event: PlayerChatAttempt) -> SjmlResult<bool> {
    Ok(event.raw_text == VETO_TEXT && !event.player_id.is_empty() && !event.player_name.is_empty())
}
