use sjml::{
    event,
    events::{self, EventPriority},
    player_continue_sleeping_attempt::PlayerContinueSleepingAttempt,
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("player.continue_sleeping_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.continue_sleeping_attempt")]
fn veto_sleepers_without_a_problem(event: PlayerContinueSleepingAttempt) -> SjmlResult<bool> {
    Ok(event.problem.is_empty())
}
