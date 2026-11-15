use sjml::{
    block_grow_feature_attempt::BlockGrowFeatureAttempt,
    event,
    events::{self, EventPriority},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("world.block_grow_feature_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "world.block_grow_feature_attempt")]
fn veto_growth_without_a_feature(event: BlockGrowFeatureAttempt) -> SjmlResult<bool> {
    Ok(event.feature_id.is_none())
}
