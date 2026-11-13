use std::sync::atomic::{AtomicBool, Ordering};

use sjml::{
    event,
    events::{self, BlockFarmlandTrampleAttempt, EventPriority},
    SjmlResult,
};

// The fixture has one deliberate farmland event. Validate the detached payload
// in the handler itself, then veto it so no extra manifest command is needed.
static ARMED: AtomicBool = AtomicBool::new(true);

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("block.farmland_trample_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "block.farmland_trample_attempt")]
fn veto_armed_farmland_trample(event: BlockFarmlandTrampleAttempt) -> SjmlResult<bool> {
    let matches_expected = event.dimension_id == "minecraft:overworld"
        && event.block_id == "minecraft:farmland"
        && canonical_uuid(&event.entity_id)
        && event.entity_type_id == "minecraft:pig"
        && event.fall_distance == 5.0;
    if !matches_expected {
        return Ok(false);
    }

    Ok(ARMED.swap(false, Ordering::Relaxed))
}

fn canonical_uuid(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 36
        && [8, 13, 18, 23]
            .into_iter()
            .all(|index| bytes[index] == b'-')
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| [8, 13, 18, 23].contains(&index) || byte.is_ascii_hexdigit())
}
