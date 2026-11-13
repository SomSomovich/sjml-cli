//! Bounded keyframe rotations for an already registered entity or block-entity cuboid model.
//!
//! The host evaluates these tracks from Minecraft tick time. Guest code is not called while rendering.

use crate::registry::ModelAnimationAxis;
use crate::{ErrorCode, SjmlError, SjmlResult};

const MODEL_KEYFRAME_SCHEMA_VERSION: u32 = 1;
const MODEL_TRANSFORM_KEYFRAME_SCHEMA_VERSION: u32 = 1;
const MAX_MODEL_PATH_BYTES: usize = 256;
const MAX_MODEL_CUBOIDS: usize = 32;
pub const MAX_MODEL_KEYFRAMES: usize = 32;
pub const MAX_MODEL_KEYFRAME_DURATION_TICKS: u32 = 12_000;
const MAX_MODEL_KEYFRAME_TRACK_BYTES: usize = 36 + MAX_MODEL_KEYFRAMES * 8;
const MAX_MODEL_TRANSFORM_KEYFRAME_TRACK_BYTES: usize = 36 + MAX_MODEL_KEYFRAMES * 16;
pub const MIN_MODEL_SCALE: f32 = 0.01;
pub const MAX_MODEL_SCALE: f32 = 16.0;

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

/// The vector transform animated by a transform keyframe track.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ModelTransformKind {
    /// Moves the model or cuboid in model-local coordinates.
    Translation = 0,
    /// Scales the model or cuboid around the track's pivot.
    Scale = 1,
}

/// One XYZ translation/scale keyframe, measured in ticks from the start of a loop.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModelTransformKeyframe {
    tick: u32,
    value: [f32; 3],
}

impl ModelTransformKeyframe {
    /// Creates a finite XYZ keyframe. Bounds depend on the selected transform track.
    pub fn new(tick: u32, value: [f32; 3]) -> SjmlResult<Self> {
        if value.iter().any(|component| !component.is_finite()) {
            return Err(invalid_argument(
                "model transform keyframe values must be finite",
            ));
        }
        Ok(Self { tick, value })
    }

    pub const fn tick(self) -> u32 {
        self.tick
    }

    pub const fn value(self) -> [f32; 3] {
        self.value
    }
}

/// One bounded periodic XYZ translation or scale track for a model part.
///
/// Translation values use model-local coordinates, bounded to +/-4 for entity models and +/-1
/// for block-entity models. Scale values are positive factors in `0.01..=16`. Keyframes are
/// linearly interpolated and the last segment wraps from the final frame to tick zero.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelTransformKeyframeTrack {
    kind: ModelTransformKind,
    pivot: [f32; 3],
    duration_ticks: u32,
    keyframes: Vec<ModelTransformKeyframe>,
}

impl ModelTransformKeyframeTrack {
    /// Creates a translation track. Translation is applied in model-local coordinates.
    pub fn translation(
        duration_ticks: u32,
        keyframes: Vec<ModelTransformKeyframe>,
    ) -> SjmlResult<Self> {
        Self::new(
            ModelTransformKind::Translation,
            [0.0; 3],
            duration_ticks,
            keyframes,
        )
    }

    /// Creates a scale track around a model-local pivot.
    pub fn scale(
        pivot: [f32; 3],
        duration_ticks: u32,
        keyframes: Vec<ModelTransformKeyframe>,
    ) -> SjmlResult<Self> {
        Self::new(ModelTransformKind::Scale, pivot, duration_ticks, keyframes)
    }

    fn new(
        kind: ModelTransformKind,
        pivot: [f32; 3],
        duration_ticks: u32,
        keyframes: Vec<ModelTransformKeyframe>,
    ) -> SjmlResult<Self> {
        if pivot
            .iter()
            .any(|coordinate| !coordinate.is_finite() || coordinate.abs() > 4.0)
        {
            return Err(invalid_argument(
                "model scale keyframe pivot must be finite and within model coordinates +/-4",
            ));
        }
        if !(2..=MAX_MODEL_KEYFRAME_DURATION_TICKS).contains(&duration_ticks) {
            return Err(invalid_argument(
                "model transform keyframe duration must be between 2 and 12000 ticks",
            ));
        }
        if keyframes.len() < 2 || keyframes.len() > MAX_MODEL_KEYFRAMES {
            return Err(invalid_argument(
                "model transform keyframe track must contain from 2 to 32 keyframes",
            ));
        }
        if keyframes[0].tick != 0 {
            return Err(invalid_argument(
                "model transform keyframe track must start at tick zero",
            ));
        }
        let mut previous_tick = None;
        for keyframe in &keyframes {
            if keyframe.tick >= duration_ticks
                || previous_tick.is_some_and(|previous| keyframe.tick <= previous)
            {
                return Err(invalid_argument(
                    "model transform keyframe ticks must increase within the loop duration",
                ));
            }
            for value in keyframe.value {
                if !value.is_finite()
                    || match kind {
                        ModelTransformKind::Translation => !(-4.0..=4.0).contains(&value),
                        ModelTransformKind::Scale => {
                            !(MIN_MODEL_SCALE..=MAX_MODEL_SCALE).contains(&value)
                        }
                    }
                {
                    return Err(invalid_argument(match kind {
                        ModelTransformKind::Translation => {
                            "model translation keyframe values must be within model coordinates +/-4"
                        }
                        ModelTransformKind::Scale => "model scale factors must be within 0.01..=16",
                    }));
                }
            }
            previous_tick = Some(keyframe.tick);
        }
        if kind == ModelTransformKind::Translation && pivot != [0.0; 3] {
            return Err(invalid_argument(
                "translation keyframes do not have a pivot",
            ));
        }
        Ok(Self {
            kind,
            pivot,
            duration_ticks,
            keyframes,
        })
    }

    pub const fn kind(&self) -> ModelTransformKind {
        self.kind
    }

    pub const fn pivot(&self) -> [f32; 3] {
        self.pivot
    }

    pub const fn duration_ticks(&self) -> u32 {
        self.duration_ticks
    }

    pub fn keyframes(&self) -> &[ModelTransformKeyframe] {
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

/// Adds a translation or scale keyframe track to a previously registered entity model.
///
/// A target may have at most one translation and one scale track. Translation uses model-local
/// XYZ coordinates; scale is positive and applies around its configured pivot.
pub fn register_entity_model_transform_keyframe_track(
    path: &str,
    target: ModelAnimationTarget,
    track: ModelTransformKeyframeTrack,
) -> SjmlResult<()> {
    if track.kind == ModelTransformKind::Translation {
        validate_transform_translation(&track, 4.0)?;
    }
    register_model_transform_keyframe_track(path, target, track, 0)
}

/// Adds a translation or scale keyframe track to a previously registered block-entity model.
///
/// Translation values and scale pivots stay within the owning block's unit cube.
pub fn register_block_entity_model_transform_keyframe_track(
    path: &str,
    target: ModelAnimationTarget,
    track: ModelTransformKeyframeTrack,
) -> SjmlResult<()> {
    if track.kind == ModelTransformKind::Translation {
        validate_transform_translation(&track, 1.0)?;
    } else if track
        .pivot
        .iter()
        .any(|coordinate| !(0.0..=1.0).contains(coordinate))
    {
        return Err(invalid_argument(
            "block-entity scale keyframe pivot must be within the block unit cube",
        ));
    }
    register_model_transform_keyframe_track(path, target, track, 1)
}

fn validate_transform_translation(
    track: &ModelTransformKeyframeTrack,
    limit: f32,
) -> SjmlResult<()> {
    if track
        .keyframes
        .iter()
        .any(|frame| frame.value.iter().any(|value| value.abs() > limit))
    {
        return Err(invalid_argument(if limit == 1.0 {
            "block-entity translation values must be within +/-1 model coordinates"
        } else {
            "entity translation values must be within +/-4 model coordinates"
        }));
    }
    Ok(())
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

#[cfg(test)]
mod transform_tests {
    use super::*;

    fn frame(tick: u32, value: [f32; 3]) -> ModelTransformKeyframe {
        ModelTransformKeyframe::new(tick, value).unwrap()
    }

    #[test]
    fn translation_and_scale_tracks_enforce_finite_bounded_periodic_data() {
        assert!(ModelTransformKeyframeTrack::translation(
            40,
            vec![frame(0, [0.0, 0.0, 0.0]), frame(20, [4.01, 0.0, 0.0])],
        )
        .is_err());
        assert!(ModelTransformKeyframeTrack::translation(
            40,
            vec![frame(1, [0.0, 0.0, 0.0]), frame(20, [1.0, 0.0, 0.0])],
        )
        .is_err());
        assert!(ModelTransformKeyframe::new(3, [f32::NAN, 0.0, 0.0]).is_err());
        assert!(ModelTransformKeyframeTrack::scale(
            [0.0, 0.5, 0.0],
            40,
            vec![frame(0, [1.0, 1.0, 1.0]), frame(20, [0.0, 1.0, 1.0])],
        )
        .is_err());
        assert!(ModelTransformKeyframeTrack::scale(
            [f32::INFINITY, 0.5, 0.0],
            40,
            vec![frame(0, [1.0, 1.0, 1.0]), frame(20, [2.0, 1.0, 1.0])],
        )
        .is_err());
        assert!(ModelTransformKeyframeTrack::translation(
            40,
            vec![frame(0, [0.0, 0.0, 0.0]), frame(40, [1.0, 0.0, 0.0])],
        )
        .is_err());
    }

    #[test]
    fn block_entity_transforms_stay_inside_the_block_space_bounds() {
        let outside_translation = ModelTransformKeyframeTrack::translation(
            40,
            vec![frame(0, [0.0, 0.0, 0.0]), frame(20, [1.01, 0.0, 0.0])],
        )
        .unwrap();
        assert!(register_block_entity_model_transform_keyframe_track(
            "machine",
            ModelAnimationTarget::Model,
            outside_translation,
        )
        .is_err());

        let outside_pivot = ModelTransformKeyframeTrack::scale(
            [0.5, 1.01, 0.5],
            40,
            vec![frame(0, [1.0, 1.0, 1.0]), frame(20, [2.0, 1.0, 1.0])],
        )
        .unwrap();
        assert!(register_block_entity_model_transform_keyframe_track(
            "machine",
            ModelAnimationTarget::Model,
            outside_pivot,
        )
        .is_err());
    }

    #[test]
    fn transform_payload_is_additive_and_encodes_xyz_values_little_endian() {
        let track = ModelTransformKeyframeTrack::scale(
            [0.25, 0.5, 0.75],
            40,
            vec![frame(0, [1.0, 1.0, 1.0]), frame(20, [2.0, 1.5, 0.5])],
        )
        .unwrap();
        let encoded = encode_model_transform_keyframe_track(&track, 0, 1);
        assert_eq!(encoded.len(), 68);
        assert_eq!(u32::from_le_bytes(encoded[0..4].try_into().unwrap()), 1);
        assert_eq!(u32::from_le_bytes(encoded[4..8].try_into().unwrap()), 0);
        assert_eq!(u32::from_le_bytes(encoded[8..12].try_into().unwrap()), 1);
        assert_eq!(u32::from_le_bytes(encoded[12..16].try_into().unwrap()), 1);
        assert_eq!(u32::from_le_bytes(encoded[28..32].try_into().unwrap()), 40);
        assert_eq!(u32::from_le_bytes(encoded[32..36].try_into().unwrap()), 2);
        assert_eq!(u32::from_le_bytes(encoded[36..40].try_into().unwrap()), 0);
        assert_eq!(
            f32::from_bits(u32::from_le_bytes(encoded[40..44].try_into().unwrap())),
            1.0
        );
        assert_eq!(u32::from_le_bytes(encoded[52..56].try_into().unwrap()), 20);
        assert_eq!(
            f32::from_bits(u32::from_le_bytes(encoded[56..60].try_into().unwrap())),
            2.0
        );
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

fn register_model_transform_keyframe_track(
    path: &str,
    target: ModelAnimationTarget,
    track: ModelTransformKeyframeTrack,
    target_kind: u32,
) -> SjmlResult<()> {
    validate_path(path)?;
    let cuboid_index = match target {
        ModelAnimationTarget::Model => u32::MAX,
        ModelAnimationTarget::Cuboid(index) if index < MAX_MODEL_CUBOIDS => index as u32,
        ModelAnimationTarget::Cuboid(_) => {
            return Err(invalid_argument(
                "model transform keyframe cuboid index must be below 32",
            ));
        }
    };
    let encoded = encode_model_transform_keyframe_track(&track, target_kind, cuboid_index);
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies both bounded input slices before this import returns.
        let status = unsafe {
            imported_register_model_transform_keyframe_track(
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
                "host rejected the model transform keyframe track",
            ))
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = encoded;
        Err(SjmlError::new(
            ErrorCode::UnknownErrorCode,
            "model transform keyframe imports require the wasm32-unknown-unknown target",
        ))
    }
}

fn encode_model_transform_keyframe_track(
    track: &ModelTransformKeyframeTrack,
    target_kind: u32,
    cuboid_index: u32,
) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(36 + track.keyframes.len() * 16);
    encoded.extend_from_slice(&MODEL_TRANSFORM_KEYFRAME_SCHEMA_VERSION.to_le_bytes());
    encoded.extend_from_slice(&target_kind.to_le_bytes());
    encoded.extend_from_slice(&cuboid_index.to_le_bytes());
    encoded.extend_from_slice(&(track.kind as u32).to_le_bytes());
    for coordinate in track.pivot {
        encoded.extend_from_slice(&coordinate.to_bits().to_le_bytes());
    }
    encoded.extend_from_slice(&track.duration_ticks.to_le_bytes());
    encoded.extend_from_slice(&(track.keyframes.len() as u32).to_le_bytes());
    for keyframe in &track.keyframes {
        encoded.extend_from_slice(&keyframe.tick.to_le_bytes());
        for value in keyframe.value {
            encoded.extend_from_slice(&value.to_bits().to_le_bytes());
        }
    }
    debug_assert!(encoded.len() <= MAX_MODEL_TRANSFORM_KEYFRAME_TRACK_BYTES);
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
    #[link_name = "register_model_transform_keyframe_track"]
    fn imported_register_model_transform_keyframe_track(
        path_pointer: i32,
        path_length: i32,
        payload_pointer: i32,
        payload_length: i32,
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
