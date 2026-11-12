//! Bounded keyframe rotations for an already registered entity or block-entity cuboid model.
//!
//! The host evaluates these tracks from Minecraft tick time. Guest code is not called while rendering.

use crate::registry::ModelAnimationAxis;
use crate::{ErrorCode, SjmlError, SjmlResult};

const MODEL_KEYFRAME_SCHEMA_VERSION: u32 = 1;
const MAX_MODEL_PATH_BYTES: usize = 256;
const MAX_MODEL_CUBOIDS: usize = 32;
pub const MAX_MODEL_KEYFRAMES: usize = 32;
pub const MAX_MODEL_KEYFRAME_DURATION_TICKS: u32 = 12_000;
const MAX_MODEL_KEYFRAME_TRACK_BYTES: usize = 36 + MAX_MODEL_KEYFRAMES * 8;

/// Selects the whole model or one of its zero-based cuboid indices as an animation target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelAnimationTarget {
    Model,
    Cuboid(usize),
}

/// One linear-interpolation keyframe, measured in ticks from the start of a loop.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModelKeyframe {
    tick: u32,
    angle_degrees: f32,
}

impl ModelKeyframe {
    /// Creates a keyframe with an angle between -180 and 180 degrees.
    pub fn new(tick: u32, angle_degrees: f32) -> SjmlResult<Self> {
        if !angle_degrees.is_finite() || !(-180.0..=180.0).contains(&angle_degrees) {
            return Err(invalid_argument(
                "model keyframe angle must be finite and within +/-180 degrees",
            ));
        }
        Ok(Self {
            tick,
            angle_degrees,
        })
    }

    pub const fn tick(self) -> u32 {
        self.tick
    }

    pub const fn angle_degrees(self) -> f32 {
        self.angle_degrees
    }
}

/// A periodic rotation track with linear interpolation between its ordered keyframes.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelKeyframeTrack {
    axis: ModelAnimationAxis,
    pivot: [f32; 3],
    duration_ticks: u32,
    keyframes: Vec<ModelKeyframe>,
}

impl ModelKeyframeTrack {
    /// Creates a periodic keyframe rotation. The first keyframe must be at tick zero;
    /// all later keyframes must occur before the loop duration.
    pub fn new(
        axis: ModelAnimationAxis,
        pivot: [f32; 3],
        duration_ticks: u32,
        keyframes: Vec<ModelKeyframe>,
    ) -> SjmlResult<Self> {
        if pivot
            .iter()
            .any(|coordinate| !coordinate.is_finite() || coordinate.abs() > 4.0)
        {
            return Err(invalid_argument(
                "model keyframe pivot must be finite and within model coordinates +/-4",
            ));
        }
        if !(2..=MAX_MODEL_KEYFRAME_DURATION_TICKS).contains(&duration_ticks) {
            return Err(invalid_argument(
                "model keyframe duration must be between 2 and 12000 ticks",
            ));
        }
        if keyframes.len() < 2 || keyframes.len() > MAX_MODEL_KEYFRAMES {
            return Err(invalid_argument(
                "model keyframe track must contain from 2 to 32 keyframes",
            ));
        }
        if keyframes[0].tick != 0 {
            return Err(invalid_argument(
                "model keyframe track must start at tick zero",
            ));
        }
        let mut previous_tick = None;
        for keyframe in &keyframes {
            if keyframe.tick >= duration_ticks
                || previous_tick.is_some_and(|previous| keyframe.tick <= previous)
            {
                return Err(invalid_argument(
                    "model keyframe ticks must increase within the loop duration",
                ));
            }
            if !keyframe.angle_degrees.is_finite()
                || !(-180.0..=180.0).contains(&keyframe.angle_degrees)
            {
                return Err(invalid_argument(
                    "model keyframe angle must be finite and within +/-180 degrees",
                ));
            }
            previous_tick = Some(keyframe.tick);
        }
        Ok(Self {
            axis,
            pivot,
            duration_ticks,
            keyframes,
        })
    }

    pub const fn axis(&self) -> ModelAnimationAxis {
        self.axis
    }

    pub const fn pivot(&self) -> [f32; 3] {
        self.pivot
    }

    pub const fn duration_ticks(&self) -> u32 {
        self.duration_ticks
    }

    pub fn keyframes(&self) -> &[ModelKeyframe] {
        &self.keyframes
    }
}

/// Adds a keyframe track to an entity model previously registered by this module.
///
/// Call from `#[mod_register]` after `register_entity_model`. A model can have one track per axis
/// on its root and on each cuboid. The host rejects tracks that target a missing model or cuboid.
pub fn register_entity_model_keyframe_track(
    path: &str,
    target: ModelAnimationTarget,
    track: ModelKeyframeTrack,
) -> SjmlResult<()> {
    register_model_keyframe_track(path, target, track, 0)
}

/// Adds a keyframe track to a block-entity model previously registered by this module.
///
/// The pivot must stay within the owning block's unit cube. Call from `#[mod_register]` after
/// `register_block_entity_model`.
pub fn register_block_entity_model_keyframe_track(
    path: &str,
    target: ModelAnimationTarget,
    track: ModelKeyframeTrack,
) -> SjmlResult<()> {
    if track
        .pivot
        .iter()
        .any(|coordinate| !(0.0..=1.0).contains(coordinate))
    {
        return Err(invalid_argument(
            "block-entity keyframe pivot must be within the block unit cube",
        ));
    }
    register_model_keyframe_track(path, target, track, 1)
}

fn register_model_keyframe_track(
    path: &str,
    target: ModelAnimationTarget,
    track: ModelKeyframeTrack,
    target_kind: u32,
) -> SjmlResult<()> {
    validate_path(path)?;
    let cuboid_index = match target {
        ModelAnimationTarget::Model => u32::MAX,
        ModelAnimationTarget::Cuboid(index) if index < MAX_MODEL_CUBOIDS => index as u32,
        ModelAnimationTarget::Cuboid(_) => {
            return Err(invalid_argument(
                "model keyframe cuboid index must be below 32",
            ));
        }
    };
    let encoded = encode_model_keyframe_track(&track, target_kind, cuboid_index);
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies both bounded input slices before this import returns.
        let status = unsafe {
            imported_register_model_keyframe_track(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as u32 as i32,
            )
        };
        if status == 0 {
            Ok(())
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected the model keyframe track",
            ))
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = encoded;
        Err(SjmlError::new(
            ErrorCode::UnknownErrorCode,
            "model keyframe imports require the wasm32-unknown-unknown target",
        ))
    }
}

fn encode_model_keyframe_track(
    track: &ModelKeyframeTrack,
    target_kind: u32,
    cuboid_index: u32,
) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(36 + track.keyframes.len() * 8);
    encoded.extend_from_slice(&MODEL_KEYFRAME_SCHEMA_VERSION.to_le_bytes());
    encoded.extend_from_slice(&target_kind.to_le_bytes());
    encoded.extend_from_slice(&cuboid_index.to_le_bytes());
    encoded.extend_from_slice(&(track.axis as u32).to_le_bytes());
    for coordinate in track.pivot {
        encoded.extend_from_slice(&coordinate.to_bits().to_le_bytes());
    }
    encoded.extend_from_slice(&track.duration_ticks.to_le_bytes());
    encoded.extend_from_slice(&(track.keyframes.len() as u32).to_le_bytes());
    for keyframe in &track.keyframes {
        encoded.extend_from_slice(&keyframe.tick.to_le_bytes());
        encoded.extend_from_slice(&keyframe.angle_degrees.to_bits().to_le_bytes());
    }
    debug_assert!(encoded.len() <= MAX_MODEL_KEYFRAME_TRACK_BYTES);
    encoded
}

fn validate_path(path: &str) -> SjmlResult<()> {
    if path.is_empty()
        || path.len() > MAX_MODEL_PATH_BYTES
        || !path.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_.-".contains(&byte)
        })
    {
        return Err(invalid_argument(
            "model path must contain 1..=256 lowercase ASCII letters, digits, '.', '_' or '-'",
        ));
    }
    Ok(())
}

fn invalid_argument(message: &str) -> SjmlError {
    SjmlError::new(ErrorCode::InvalidArgument, message)
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_registry_v1")]
unsafe extern "C" {
    #[link_name = "register_model_keyframe_track"]
    fn imported_register_model_keyframe_track(
        path_pointer: i32,
        path_length: i32,
        track_pointer: i32,
        track_length: i32,
    ) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn track(keyframes: Vec<ModelKeyframe>) -> ModelKeyframeTrack {
        ModelKeyframeTrack::new(ModelAnimationAxis::Y, [0.0, 0.5, 0.0], 40, keyframes).unwrap()
    }

    #[test]
    fn keyframe_track_requires_bounded_ordered_loop_frames() {
        assert!(ModelKeyframeTrack::new(
            ModelAnimationAxis::Y,
            [0.0, 0.5, 0.0],
            40,
            vec![
                ModelKeyframe::new(0, 0.0).unwrap(),
                ModelKeyframe::new(40, 90.0).unwrap()
            ],
        )
        .is_err());
        assert!(ModelKeyframeTrack::new(
            ModelAnimationAxis::Y,
            [0.0, 0.5, 0.0],
            40,
            vec![
                ModelKeyframe::new(1, 0.0).unwrap(),
                ModelKeyframe::new(20, 90.0).unwrap()
            ],
        )
        .is_err());
        assert!(ModelKeyframe::new(3, f32::NAN).is_err());
        assert!(ModelKeyframe::new(3, 181.0).is_err());
        assert!(ModelKeyframeTrack::new(
            ModelAnimationAxis::X,
            [f32::INFINITY, 0.0, 0.0],
            40,
            vec![
                ModelKeyframe::new(0, 0.0).unwrap(),
                ModelKeyframe::new(20, 0.0).unwrap()
            ],
        )
        .is_err());
    }

    #[test]
    fn keyframe_payload_has_bounded_schema_and_little_endian_frames() {
        let value = track(vec![
            ModelKeyframe::new(0, -12.5).unwrap(),
            ModelKeyframe::new(13, 60.0).unwrap(),
        ]);
        let encoded = encode_model_keyframe_track(&value, 0, 2);
        assert_eq!(encoded.len(), 52);
        assert_eq!(u32::from_le_bytes(encoded[0..4].try_into().unwrap()), 1);
        assert_eq!(u32::from_le_bytes(encoded[4..8].try_into().unwrap()), 0);
        assert_eq!(u32::from_le_bytes(encoded[8..12].try_into().unwrap()), 2);
        assert_eq!(u32::from_le_bytes(encoded[28..32].try_into().unwrap()), 40);
        assert_eq!(u32::from_le_bytes(encoded[32..36].try_into().unwrap()), 2);
        assert_eq!(u32::from_le_bytes(encoded[36..40].try_into().unwrap()), 0);
        assert_eq!(
            f32::from_bits(u32::from_le_bytes(encoded[40..44].try_into().unwrap())),
            -12.5
        );
    }
}
