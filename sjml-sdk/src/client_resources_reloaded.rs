//! Physical-client notification emitted after a resource reload is fully applied.
//!
//! Subscribe with `events::subscribe("client.resources_reloaded", EventPriority::Normal)` and
//! declare a typed `#[event_async(name = "client.resources_reloaded")]` handler. The host calls
//! the export on the client tick only after Minecraft has removed its loading overlay and the
//! active resource-pack set matches the set observed by NeoForge's reload listener. The callback
//! receives no Minecraft objects and carries a positive, monotonic generation ID only.

use crate::serialization::{Decoder, Encoder};
use crate::{ErrorCode, SjmlError, SjmlResult, StableDecode, StableEncode};

/// Stable event name used with [`crate::events::subscribe`].
pub const EVENT_NAME: &str = "client.resources_reloaded";
/// Maximum generation representable by the Java host without wrapping a signed `long`.
pub const MAX_GENERATION: u64 = i64::MAX as u64;

/// Detached notification for one successfully completed physical-client resource reload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientResourcesReloaded {
    /// Positive monotonic ID, starting at 1 for the first observed successful client reload.
    pub generation: u64,
}

impl StableEncode for ClientResourcesReloaded {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        validate_generation(self.generation)?;
        encoder.write_u32(1)?;
        encoder.write_u64(self.generation)
    }
}

impl StableDecode for ClientResourcesReloaded {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        let version = decoder.read_u32()?;
        if version != 1 {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                format!("ClientResourcesReloaded schema version {version} is unsupported"),
            ));
        }
        let event = Self {
            generation: decoder.read_u64()?,
        };
        validate_generation(event.generation)?;
        Ok(event)
    }
}

fn validate_generation(generation: u64) -> SjmlResult<()> {
    if !(1..=MAX_GENERATION).contains(&generation) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "client resource reload generation must be in 1..=i64::MAX",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ClientResourcesReloaded, MAX_GENERATION};
    use crate::serialization::{from_bytes, to_bytes};
    use crate::{ErrorCode, StableDecode};

    #[test]
    fn stable_round_trip_accepts_positive_bounded_generation() {
        for generation in [1, 42, MAX_GENERATION] {
            let event = ClientResourcesReloaded { generation };
            assert_eq!(
                from_bytes::<ClientResourcesReloaded>(&to_bytes(&event).unwrap()).unwrap(),
                event
            );
        }
    }

    #[test]
    fn stable_codec_rejects_zero_and_values_above_host_bound() {
        for generation in [0, MAX_GENERATION + 1] {
            let event = ClientResourcesReloaded { generation };
            assert_eq!(
                to_bytes(&event).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
            let mut raw = Vec::new();
            raw.extend_from_slice(&1_u32.to_le_bytes());
            raw.extend_from_slice(&generation.to_le_bytes());
            assert_eq!(
                ClientResourcesReloaded::decode(
                    &mut crate::serialization::Decoder::new(&raw).unwrap()
                )
                .unwrap_err()
                .code(),
                ErrorCode::InvalidArgument
            );
        }
    }

    #[test]
    fn stable_codec_rejects_unsupported_schema() {
        let mut raw = 2_u32.to_le_bytes().to_vec();
        raw.extend_from_slice(&1_u64.to_le_bytes());
        assert_eq!(
            from_bytes::<ClientResourcesReloaded>(&raw)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );
    }
}
