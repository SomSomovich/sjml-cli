use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering};

use sjml::{
    command, event,
    events::{self, BlockGrowAttempt, EventPriority},
    ErrorCode, SjmlError, SjmlResult, StableDecode,
};

static ARMED: AtomicBool = AtomicBool::new(false);
static EXPECTED_X: AtomicI32 = AtomicI32::new(0);
static EXPECTED_Y: AtomicI32 = AtomicI32::new(0);
static EXPECTED_Z: AtomicI32 = AtomicI32::new(0);
static MATCHING_CALLBACKS: AtomicU32 = AtomicU32::new(0);
static SAW_VALID_SNAPSHOT: AtomicBool = AtomicBool::new(false);

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("block.grow_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "block.grow_attempt")]
fn veto_armed_wheat_growth(event: BlockGrowAttempt) -> SjmlResult<bool> {
    let position = event.position;
    let matches_expected = position.x == EXPECTED_X.load(Ordering::Relaxed)
        && position.y == EXPECTED_Y.load(Ordering::Relaxed)
        && position.z == EXPECTED_Z.load(Ordering::Relaxed)
        && event.block_id == "minecraft:wheat"
        && event.dimension_id == "minecraft:overworld";
    if !matches_expected {
        return Ok(false);
    }

    MATCHING_CALLBACKS.fetch_add(1, Ordering::Relaxed);
    let valid = ARMED.load(Ordering::Relaxed);
    SAW_VALID_SNAPSHOT.store(valid, Ordering::Relaxed);
    Ok(valid && ARMED.swap(false, Ordering::Relaxed))
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct PositionArguments {
    x: i32,
    y: i32,
    z: i32,
}

#[command(name = "block.grow_attempt.arm")]
fn arm_crop_growth(arguments: PositionArguments) -> SjmlResult<String> {
    EXPECTED_X.store(arguments.x, Ordering::Relaxed);
    EXPECTED_Y.store(arguments.y, Ordering::Relaxed);
    EXPECTED_Z.store(arguments.z, Ordering::Relaxed);
    MATCHING_CALLBACKS.store(0, Ordering::Relaxed);
    SAW_VALID_SNAPSHOT.store(false, Ordering::Relaxed);
    ARMED.store(true, Ordering::Relaxed);
    Ok("block growth veto armed".to_owned())
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct NoArguments {}

#[command(name = "block.grow_attempt.check")]
fn check_crop_growth_veto(_: NoArguments) -> SjmlResult<String> {
    let callbacks = MATCHING_CALLBACKS.swap(0, Ordering::Relaxed);
    if callbacks == 1
        && !ARMED.load(Ordering::Relaxed)
        && SAW_VALID_SNAPSHOT.swap(false, Ordering::Relaxed)
    {
        Ok("guest received one detached block-growth attempt and vetoed it".to_owned())
    } else {
        Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "block.grow_attempt did not deliver one valid callback and veto",
        ))
    }
}
