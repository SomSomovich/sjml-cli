#![cfg_attr(
    not(all(target_arch = "wasm32", target_os = "unknown")),
    allow(dead_code)
)]

use sjml::{
    event,
    events::{self, BlockDropsAttempt, EventPriority},
    mod_init, SjmlResult,
};

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("block.drops_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "block.drops_attempt")]
fn suppress_sample_ore_drops(attempt: BlockDropsAttempt) -> SjmlResult<bool> {
    // Replace this example identifier with the block whose computed drops should be suppressed.
    Ok(attempt.block_id == "example:sample_ore")
}
