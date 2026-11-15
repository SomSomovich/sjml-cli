//! Detached payload and veto response for logical-server fluid source formation.
//!
//! Register a `world.fluid_source_attempt` callback with `events.subscribe` and `events.cancel`.
//! Returning `Ok(true)` sets the NeoForge event's `canConvert` flag to `false`, so the fluid is
//! not converted into an infinite source block. The callback is synchronous because vanilla reads
//! the flag as soon as the event returns, so `#[event_async]` handlers are rejected.

use crate::{StableDecode, StableEncode};

/// Event name for synchronous logical-server fluid source attempts.
pub const EVENT_NAME: &str = "world.fluid_source_attempt";
/// Guest export name the host resolves for one fluid source callback.
pub const EXPORT_NAME: &str = "sjml_v1_event_world.fluid_source_attempt";
/// Cancellation grant required to veto source formation; `events.subscribe` is also required.
pub const CANCEL_CAPABILITY: &str = "events.cancel";
/// Schema version of the detached fluid source payload.
pub const SCHEMA_VERSION: u32 = 1;
/// Maximum UTF-8 byte length of a dimension ID or fluid ID.
pub const MAX_STRING_BYTES: usize = 256;
/// Maximum encoded payload size for one fluid source attempt callback.
pub const MAX_PAYLOAD_BYTES: usize = 1_024;
/// Maximum callbacks the host invokes for one fluid source attempt.
pub const MAX_CALLBACKS: usize = 64;
/// Maximum absolute block x/z coordinate accepted in a fluid source attempt.
pub const MAX_BLOCK_COORDINATE: i32 = 30_000_000;

/// Synchronous snapshot captured when a fluid checks whether it can become a source.
///
/// NeoForge fires `world.fluid_source_attempt` through `CreateFluidSourceEvent` whenever a
/// flowing fluid checks nearby blocks for infinite-source formation. Return `Ok(true)` to call
/// `setCanConvert(false)`, which prevents the conversion entirely. Return `Ok(false)` to keep
/// vanilla behavior. `vanilla_result` carries the precomputed
/// `FluidState.canConvertToSource` value, so a fluid that vanilla already rejects reports
/// `false`. All values are detached; no `Level`, block position handle, or fluid object crosses
/// the guest ABI. Both `events.subscribe` and `events.cancel` are required, and the callback
/// must remain synchronous.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct WorldFluidSourceAttempt {
    /// Resource location of the dimension containing the fluid.
    pub dimension_id: String,
    /// Fluid block x coordinate, bounded to ±30,000,000.
    pub x: i32,
    /// Fluid block y coordinate.
    pub y: i32,
    /// Fluid block z coordinate, bounded to ±30,000,000.
    pub z: i32,
    /// Registry ID of the fluid, such as `minecraft:water` or `minecraft:lava`.
    pub fluid_id: String,
    /// Whether vanilla would convert the fluid into a source block.
    pub vanilla_result: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_bytes, to_bytes, ErrorCode};

    fn event() -> WorldFluidSourceAttempt {
        WorldFluidSourceAttempt {
            dimension_id: "minecraft:overworld".into(),
            x: 1,
            y: 64,
            z: -3,
            fluid_id: "minecraft:water".into(),
            vanilla_result: true,
        }
    }

    fn push_string(bytes: &mut Vec<u8>, value: &str) {
        bytes.extend_from_slice(&(value.len() as u32).to_le_bytes());
        bytes.extend_from_slice(value.as_bytes());
    }

    #[test]
    fn encodes_the_java_schema_v1_wire_bytes() {
        let event = event();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        push_string(&mut expected, &event.dimension_id);
        expected.extend_from_slice(&event.x.to_le_bytes());
        expected.extend_from_slice(&event.y.to_le_bytes());
        expected.extend_from_slice(&event.z.to_le_bytes());
        push_string(&mut expected, &event.fluid_id);
        expected.push(1);
        assert_eq!(to_bytes(&event).unwrap(), expected);
        assert_eq!(
            from_bytes::<WorldFluidSourceAttempt>(&expected).unwrap(),
            event
        );
    }

    #[test]
    fn round_trips_negative_coordinates_and_false_vanilla_result() {
        for event in [
            event(),
            WorldFluidSourceAttempt {
                x: -MAX_BLOCK_COORDINATE,
                z: MAX_BLOCK_COORDINATE,
                fluid_id: "minecraft:lava".into(),
                vanilla_result: false,
                ..event()
            },
        ] {
            let encoded = to_bytes(&event).unwrap();
            assert_eq!(
                from_bytes::<WorldFluidSourceAttempt>(&encoded).unwrap(),
                event
            );
            assert!(encoded.len() <= MAX_PAYLOAD_BYTES);
        }
    }

    #[test]
    fn rejects_other_schema_versions_and_malformed_booleans() {
        let mut wrong_schema = to_bytes(&event()).unwrap();
        wrong_schema[..4].copy_from_slice(&2_i32.to_le_bytes());
        assert_eq!(
            from_bytes::<WorldFluidSourceAttempt>(&wrong_schema)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );

        let mut wrong_boolean = to_bytes(&event()).unwrap();
        *wrong_boolean.last_mut().unwrap() = 2;
        assert_eq!(
            from_bytes::<WorldFluidSourceAttempt>(&wrong_boolean)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidBoolean
        );
    }

    #[test]
    fn constants_match_the_host_contract() {
        assert_eq!(EVENT_NAME, "world.fluid_source_attempt");
        assert_eq!(EXPORT_NAME, "sjml_v1_event_world.fluid_source_attempt");
        assert_eq!(CANCEL_CAPABILITY, "events.cancel");
        assert_eq!(SCHEMA_VERSION, 1);
        assert_eq!(MAX_STRING_BYTES, 256);
        assert_eq!(MAX_PAYLOAD_BYTES, 1_024);
        assert_eq!(MAX_CALLBACKS, 64);
        assert_eq!(MAX_BLOCK_COORDINATE, 30_000_000);
    }
}
