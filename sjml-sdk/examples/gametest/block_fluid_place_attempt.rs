use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering};

use sjml::{
    command, event,
    events::{self, BlockFluidPlaceAttempt, EventPriority},
    ErrorCode, SjmlError, SjmlResult, StableDecode,
};

static ARMED: AtomicBool = AtomicBool::new(false);
static TARGET_X: AtomicI32 = AtomicI32::new(0);
static TARGET_Y: AtomicI32 = AtomicI32::new(0);
static TARGET_Z: AtomicI32 = AtomicI32::new(0);
static LIQUID_X: AtomicI32 = AtomicI32::new(0);
static LIQUID_Y: AtomicI32 = AtomicI32::new(0);
static LIQUID_Z: AtomicI32 = AtomicI32::new(0);
static MATCHING_CALLBACKS: AtomicU32 = AtomicU32::new(0);

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("block.fluid_place_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "block.fluid_place_attempt")]
fn veto_armed_fluid_placement(event: BlockFluidPlaceAttempt) -> SjmlResult<bool> {
    let target = event.target_position;
    let liquid = event.liquid_position;
    let matches_expected = target.x == TARGET_X.load(Ordering::Relaxed)
        && target.y == TARGET_Y.load(Ordering::Relaxed)
        && target.z == TARGET_Z.load(Ordering::Relaxed)
        && liquid.x == LIQUID_X.load(Ordering::Relaxed)
        && liquid.y == LIQUID_Y.load(Ordering::Relaxed)
        && liquid.z == LIQUID_Z.load(Ordering::Relaxed)
        && event.dimension_id == "minecraft:overworld"
        && event.fluid_id == "minecraft:water"
        && event.original_block_id == "minecraft:stone"
        && event.result_block_id == "minecraft:fire";
    if !matches_expected {
        return Ok(false);
    }

    MATCHING_CALLBACKS.fetch_add(1, Ordering::Relaxed);
    Ok(ARMED.swap(false, Ordering::Relaxed))
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct PositionArguments {
    target_x: i32,
    target_y: i32,
    target_z: i32,
    liquid_x: i32,
    liquid_y: i32,
    liquid_z: i32,
}

#[command(name = "block.fluid_place_attempt.arm")]
fn arm_fluid_placement_veto(arguments: PositionArguments) -> SjmlResult<String> {
    TARGET_X.store(arguments.target_x, Ordering::Relaxed);
    TARGET_Y.store(arguments.target_y, Ordering::Relaxed);
    TARGET_Z.store(arguments.target_z, Ordering::Relaxed);
    LIQUID_X.store(arguments.liquid_x, Ordering::Relaxed);
    LIQUID_Y.store(arguments.liquid_y, Ordering::Relaxed);
    LIQUID_Z.store(arguments.liquid_z, Ordering::Relaxed);
    MATCHING_CALLBACKS.store(0, Ordering::Relaxed);
    ARMED.store(true, Ordering::Relaxed);
    Ok("fluid placement veto armed".to_owned())
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct NoArguments {}

#[command(name = "block.fluid_place_attempt.check")]
fn check_fluid_placement_veto(_: NoArguments) -> SjmlResult<String> {
    if MATCHING_CALLBACKS.swap(0, Ordering::Relaxed) == 1 && !ARMED.load(Ordering::Relaxed) {
        Ok("guest received and vetoed one matching fluid placement".to_owned())
    } else {
        Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "block.fluid_place_attempt did not deliver exactly one matching veto",
        ))
    }
}
