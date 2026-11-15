//! Detached payload for cancellable logical-server feature growth attempts.
//!
//! Register a `world.block_grow_feature_attempt` callback with `events.subscribe`
//! and `events.cancel`. Returning `Ok(true)` cancels the feature placement, for
//! example bonemeal-driven tree growth. The callback is synchronous because
//! NeoForge checks cancellation while it is still dispatching the growth event.

use crate::{StableDecode, StableEncode};

/// Event name subscribed with `events::subscribe`.
pub const BLOCK_GROW_FEATURE_ATTEMPT_EVENT_NAME: &str = "world.block_grow_feature_attempt";
/// Cancellation grant required to veto feature growth; `events.subscribe` is also required.
pub const BLOCK_GROW_FEATURE_ATTEMPT_CANCEL_CAPABILITY: &str = "events.cancel";
/// Maximum UTF-8 byte length of a dimension or feature ID.
pub const MAX_BLOCK_GROW_FEATURE_ATTEMPT_STRING_BYTES: usize = 256;
/// Maximum absolute block x/z coordinate accepted in a feature growth attempt.
pub const MAX_BLOCK_GROW_FEATURE_ATTEMPT_COORDINATE: i32 = 30_000_000;
/// Maximum serialized payload size for one feature growth attempt callback.
pub const MAX_BLOCK_GROW_FEATURE_ATTEMPT_PAYLOAD_BYTES: usize = 1_024;
/// Maximum callbacks invoked for one feature growth attempt.
pub const MAX_BLOCK_GROW_FEATURE_ATTEMPT_CALLBACKS: usize = 64;

/// Synchronous cancellable snapshot captured when a block is about to grow into a feature.
///
/// NeoForge fires `world.block_grow_feature_attempt` before vanilla places the
/// feature, such as a sapling becoming a tree. Return `Ok(true)` to cancel the
/// placement; the original block remains in place. The detached snapshot
/// contains the dimension, block position, and the canonical registry ID of
/// the configured feature, or `None` when the feature holder is absent.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct BlockGrowFeatureAttempt {
    /// Resource location of the dimension containing the growing block.
    pub dimension_id: String,
    /// Growing block x coordinate.
    pub x: i32,
    /// Growing block y coordinate.
    pub y: i32,
    /// Growing block z coordinate.
    pub z: i32,
    /// Canonical registry ID of the configured feature, or `None` when unreferenced.
    pub feature_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_bytes, to_bytes, ErrorCode};

    fn sample() -> BlockGrowFeatureAttempt {
        BlockGrowFeatureAttempt {
            dimension_id: "minecraft:overworld".into(),
            x: 1,
            y: 64,
            z: -3,
            feature_id: Some("minecraft:oak".into()),
        }
    }

    fn expected_wire(event: &BlockGrowFeatureAttempt) -> Vec<u8> {
        let mut output = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut output, &event.dimension_id);
        output.extend_from_slice(&event.x.to_le_bytes());
        output.extend_from_slice(&event.y.to_le_bytes());
        output.extend_from_slice(&event.z.to_le_bytes());
        match &event.feature_id {
            Some(feature_id) => {
                output.push(1);
                append_test_string(&mut output, feature_id);
            }
            None => output.push(0),
        }
        output
    }

    #[test]
    fn grow_feature_attempt_matches_the_java_wire_layout_and_round_trips() {
        let event = sample();
        let expected = expected_wire(&event);
        assert_eq!(to_bytes(&event).unwrap(), expected);
        assert_eq!(
            from_bytes::<BlockGrowFeatureAttempt>(&expected).unwrap(),
            event
        );
    }

    #[test]
    fn grow_feature_attempt_round_trips_an_absent_feature_and_negative_coordinates() {
        let mut event = sample();
        event.x = -30_000_000;
        event.y = -64;
        event.z = 30_000_000;
        event.feature_id = None;
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(encoded.last(), Some(&0));
        assert_eq!(
            from_bytes::<BlockGrowFeatureAttempt>(&encoded).unwrap(),
            event
        );
    }

    #[test]
    fn grow_feature_attempt_rejects_other_schema_versions() {
        let mut encoded = to_bytes(&sample()).unwrap();
        encoded[..4].copy_from_slice(&2_i32.to_le_bytes());
        assert_eq!(
            from_bytes::<BlockGrowFeatureAttempt>(&encoded)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );
    }

    #[test]
    fn grow_feature_attempt_constants_match_the_host_contract() {
        assert_eq!(
            BLOCK_GROW_FEATURE_ATTEMPT_EVENT_NAME,
            "world.block_grow_feature_attempt"
        );
        assert_eq!(
            BLOCK_GROW_FEATURE_ATTEMPT_CANCEL_CAPABILITY,
            "events.cancel"
        );
        assert_eq!(MAX_BLOCK_GROW_FEATURE_ATTEMPT_STRING_BYTES, 256);
        assert_eq!(MAX_BLOCK_GROW_FEATURE_ATTEMPT_COORDINATE, 30_000_000);
        assert_eq!(MAX_BLOCK_GROW_FEATURE_ATTEMPT_PAYLOAD_BYTES, 1_024);
        assert_eq!(MAX_BLOCK_GROW_FEATURE_ATTEMPT_CALLBACKS, 64);
    }

    fn append_test_string(output: &mut Vec<u8>, value: &str) {
        output.extend_from_slice(&(value.len() as i32).to_le_bytes());
        output.extend_from_slice(value.as_bytes());
    }
}
