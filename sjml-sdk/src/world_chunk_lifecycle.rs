//! Detached server chunk load and unload notifications.

use sjml::{Decoder, Encoder, ErrorCode, SjmlError, SjmlResult, StableDecode, StableEncode};

/// Event name emitted after a chunk enters a logical server level.
pub const WORLD_CHUNK_LOADED_EVENT_NAME: &str = "world.chunk_loaded";
/// Event name emitted while a chunk is being removed from a logical server level.
pub const WORLD_CHUNK_UNLOADED_EVENT_NAME: &str = "world.chunk_unloaded";
/// Maximum UTF-8 byte length of a dimension resource location in these callbacks.
pub const MAX_WORLD_CHUNK_DIMENSION_ID_BYTES: usize = 256;
/// Maximum absolute chunk X or Z coordinate representable by these callbacks.
pub const MAX_WORLD_CHUNK_COORDINATE: i32 = 30_000_000 >> 4;
/// Maximum callbacks the host invokes for one chunk lifecycle event.
pub const MAX_WORLD_CHUNK_LIFECYCLE_CALLBACKS: usize = 64;
/// Maximum encoded payload size for one detached chunk lifecycle callback.
pub const MAX_WORLD_CHUNK_LIFECYCLE_PAYLOAD_BYTES: usize =
    4 + 4 + MAX_WORLD_CHUNK_DIMENSION_ID_BYTES + 4 + 4;

/// Observation-only snapshot for `world.chunk_loaded` and `world.chunk_unloaded`.
///
/// The value contains no Minecraft handles and is safe to keep after the source chunk is unloaded.
/// Chunk coordinates are bounded to the block-world limits supported by Minecraft 1.21.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldChunkLifecycle {
    /// Canonical resource location of the logical-server dimension, for example
    /// `minecraft:overworld`.
    pub dimension_id: String,
    /// Chunk X coordinate.
    pub chunk_x: i32,
    /// Chunk Z coordinate.
    pub chunk_z: i32,
}

impl StableEncode for WorldChunkLifecycle {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        validate(self)?;
        encoder.write_i32(1)?;
        encode_bounded_string(&self.dimension_id, encoder)?;
        encoder.write_i32(self.chunk_x)?;
        encoder.write_i32(self.chunk_z)
    }
}

impl StableDecode for WorldChunkLifecycle {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        let version = decoder.read_i32()?;
        if version != 1 {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                format!("world chunk lifecycle schema version {version} is unsupported"),
            ));
        }
        let event = Self {
            dimension_id: decode_bounded_string(decoder)?,
            chunk_x: decoder.read_i32()?,
            chunk_z: decoder.read_i32()?,
        };
        validate(&event)?;
        Ok(event)
    }
}

fn validate(event: &WorldChunkLifecycle) -> SjmlResult<()> {
    if !canonical_resource_location(&event.dimension_id) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "world chunk lifecycle dimension ID must be a canonical resource location",
        ));
    }
    if event.dimension_id.len() > MAX_WORLD_CHUNK_DIMENSION_ID_BYTES {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "world chunk lifecycle dimension ID exceeds 256 UTF-8 bytes",
        ));
    }
    if !(-MAX_WORLD_CHUNK_COORDINATE..=MAX_WORLD_CHUNK_COORDINATE).contains(&event.chunk_x)
        || !(-MAX_WORLD_CHUNK_COORDINATE..=MAX_WORLD_CHUNK_COORDINATE).contains(&event.chunk_z)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "world chunk lifecycle coordinates are outside the supported world bounds",
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
        && !value[namespace.len() + 1..].contains(':')
        && namespace.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'.' | b'-')
        })
        && path.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'_' | b'.' | b'-' | b'/')
        })
}

fn encode_bounded_string(value: &str, encoder: &mut Encoder) -> SjmlResult<()> {
    if value.len() > MAX_WORLD_CHUNK_DIMENSION_ID_BYTES {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "world chunk lifecycle dimension ID exceeds 256 UTF-8 bytes",
        ));
    }
    encoder.write_length(value.len())?;
    encoder.write_raw(value.as_bytes())
}

fn decode_bounded_string(decoder: &mut Decoder<'_>) -> SjmlResult<String> {
    let length = decoder.read_length(MAX_WORLD_CHUNK_DIMENSION_ID_BYTES)?;
    String::from_utf8(decoder.read_raw(length)?.to_vec()).map_err(|_| {
        SjmlError::new(
            ErrorCode::InvalidUtf8,
            "world chunk lifecycle dimension ID contains malformed UTF-8",
        )
    })
}
