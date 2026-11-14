//! Deferred logical-server level-save notifications.
//!
//! `world.save_observed` is emitted from NeoForge `LevelEvent.Save` after level metadata, chunks,
//! and entities have been serialized, then queued until a later server tick. The callback is
//! observer-only and cannot affect that save. It does not mean asynchronous storage I/O completed
//! or that data is durable. The payload contains only a canonical dimension ID and game time.

use sjml::{Decoder, Encoder, ErrorCode, SjmlError, SjmlResult, StableDecode, StableEncode};

/// Event name used by `#[event_async(name = "world.save_observed")]` handlers.
pub const WORLD_SAVE_OBSERVED_EVENT_NAME: &str = "world.save_observed";
/// Maximum UTF-8 byte length of the canonical dimension resource location.
pub const MAX_WORLD_SAVE_DIMENSION_ID_BYTES: usize = 256;
/// Maximum game-time value representable by the Java host, in ticks.
pub const MAX_WORLD_SAVE_GAME_TIME_TICKS: i64 = 1_000_000_000_000;
/// Maximum module callbacks invoked for one save observation.
pub const MAX_WORLD_SAVE_CALLBACKS_PER_EVENT: usize = 64;
/// Maximum encoded size for a `world.save_observed` schema-v1 payload.
pub const MAX_WORLD_SAVE_OBSERVED_PAYLOAD_BYTES: usize =
    4 + 4 + MAX_WORLD_SAVE_DIMENSION_ID_BYTES + core::mem::size_of::<i64>();

/// Detached observation of one logical-server level-save invocation.
///
/// This may be reported repeatedly for autosaves, explicit save commands, and shutdown saves.
/// It is delivered on a later server tick and does not indicate durable completion of async I/O.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldSaveObserved {
    /// Canonical resource location for the dimension, for example `minecraft:overworld`.
    pub dimension_id: String,
    /// Game time when NeoForge reported the level save, bounded to the host's supported range.
    pub game_time: i64,
}

impl StableEncode for WorldSaveObserved {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        validate(self)?;
        encoder.write_u32(1)?;
        encoder.write_length(self.dimension_id.len())?;
        encoder.write_raw(self.dimension_id.as_bytes())?;
        encoder.write_i64(self.game_time)
    }
}

impl StableDecode for WorldSaveObserved {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        let version = decoder.read_u32()?;
        if version != 1 {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                format!("WorldSaveObserved schema version {version} is unsupported"),
            ));
        }
        let length = decoder.read_length(MAX_WORLD_SAVE_DIMENSION_ID_BYTES)?;
        let dimension_id = String::from_utf8(decoder.read_raw(length)?.to_vec()).map_err(|_| {
            SjmlError::new(
                ErrorCode::InvalidUtf8,
                "world.save_observed dimension ID contains malformed UTF-8",
            )
        })?;
        let event = Self {
            dimension_id,
            game_time: decoder.read_i64()?,
        };
        validate(&event)?;
        Ok(event)
    }
}

fn validate(event: &WorldSaveObserved) -> SjmlResult<()> {
    if event.dimension_id.len() > MAX_WORLD_SAVE_DIMENSION_ID_BYTES {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "world.save_observed dimension ID exceeds 256 UTF-8 bytes",
        ));
    }
    if !canonical_resource_location(&event.dimension_id) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "world.save_observed dimension ID must be a canonical resource location",
        ));
    }
    if !(0..=MAX_WORLD_SAVE_GAME_TIME_TICKS).contains(&event.game_time) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "world.save_observed game time must be within 0..=1000000000000 ticks",
        ));
    }
    Ok(())
}

fn canonical_resource_location(value: &str) -> bool {
    let Some((namespace, path)) = value.split_once(':') else {
        return false;
    };
    !namespace.is_empty()
        && !path.is_empty()
        && !path.contains(':')
        && namespace.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'.' | b'-')
        })
        && path.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'_' | b'.' | b'-' | b'/')
        })
}

#[cfg(test)]
mod tests {
    use super::{
        WorldSaveObserved, MAX_WORLD_SAVE_CALLBACKS_PER_EVENT, MAX_WORLD_SAVE_DIMENSION_ID_BYTES,
        MAX_WORLD_SAVE_GAME_TIME_TICKS, MAX_WORLD_SAVE_OBSERVED_PAYLOAD_BYTES,
        WORLD_SAVE_OBSERVED_EVENT_NAME,
    };
    use sjml::{from_bytes, to_bytes, Decoder, ErrorCode, StableDecode};

    #[test]
    fn event_contract_and_stable_codec_round_trip() {
        assert_eq!(WORLD_SAVE_OBSERVED_EVENT_NAME, "world.save_observed");
        assert_eq!(MAX_WORLD_SAVE_CALLBACKS_PER_EVENT, 64);
        for game_time in [0, 42, MAX_WORLD_SAVE_GAME_TIME_TICKS] {
            let event = WorldSaveObserved {
                dimension_id: "minecraft:overworld".to_owned(),
                game_time,
            };
            let encoded = to_bytes(&event).unwrap();
            assert!(encoded.len() <= MAX_WORLD_SAVE_OBSERVED_PAYLOAD_BYTES);
            assert_eq!(from_bytes::<WorldSaveObserved>(&encoded).unwrap(), event);
        }
    }

    #[test]
    fn encoder_rejects_noncanonical_oversized_and_out_of_range_values() {
        for event in [
            WorldSaveObserved {
                dimension_id: "Minecraft:overworld".to_owned(),
                game_time: 1,
            },
            WorldSaveObserved {
                dimension_id: "minecraft:overworld".to_owned(),
                game_time: -1,
            },
            WorldSaveObserved {
                dimension_id: "minecraft:overworld".to_owned(),
                game_time: MAX_WORLD_SAVE_GAME_TIME_TICKS + 1,
            },
            WorldSaveObserved {
                dimension_id: format!("x:{}", "a".repeat(MAX_WORLD_SAVE_DIMENSION_ID_BYTES)),
                game_time: 1,
            },
        ] {
            assert!(to_bytes(&event).is_err());
        }
    }

    #[test]
    fn decoder_rejects_unknown_schema_invalid_utf8_and_trailing_bytes() {
        let mut unknown_schema = Vec::new();
        unknown_schema.extend_from_slice(&2_u32.to_le_bytes());
        assert_eq!(
            from_bytes::<WorldSaveObserved>(&unknown_schema)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );

        let malformed_utf8 = [
            1_u32.to_le_bytes().as_slice(),
            1_u32.to_le_bytes().as_slice(),
            &[0xff],
            1_i64.to_le_bytes().as_slice(),
        ]
        .concat();
        assert_eq!(
            from_bytes::<WorldSaveObserved>(&malformed_utf8)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidUtf8
        );

        let valid = to_bytes(&WorldSaveObserved {
            dimension_id: "minecraft:overworld".to_owned(),
            game_time: 1,
        })
        .unwrap();
        let mut trailing = valid;
        trailing.push(0);
        assert!(from_bytes::<WorldSaveObserved>(&trailing).is_err());

        let mut decoder = Decoder::new(&malformed_utf8).unwrap();
        assert_eq!(
            WorldSaveObserved::decode(&mut decoder).unwrap_err().code(),
            ErrorCode::InvalidUtf8
        );
    }
}
