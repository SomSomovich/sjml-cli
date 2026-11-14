//! Detached observation of an entity leaving one logical server level.

use sjml::{Decoder, Encoder, ErrorCode, SjmlError, SjmlResult, StableDecode, StableEncode};

/// Event name emitted when NeoForge reports that an entity left a logical server level.
pub const ENTITY_LEFT_LEVEL_EVENT_NAME: &str = "entity.left_level";
/// Maximum UTF-8 byte length of entity type and dimension resource locations.
pub const MAX_ENTITY_LEFT_LEVEL_STRING_BYTES: usize = 256;
/// Maximum callbacks the host invokes for one entity level-leave event.
pub const MAX_ENTITY_LEFT_LEVEL_CALLBACKS: usize = 64;
/// Maximum encoded payload size for one detached entity level-leave callback.
pub const MAX_ENTITY_LEFT_LEVEL_PAYLOAD_BYTES: usize =
    4 + 3 * 4 + 36 + 2 * MAX_ENTITY_LEFT_LEVEL_STRING_BYTES + 3 * 8;

/// Observation-only snapshot that an entity left one particular logical server level.
///
/// The notification can result from a dimension transfer, chunk unload, or other level removal.
/// It does not say that the entity died or despawned. All values are detached; no `Entity`, level,
/// or host handle crosses the guest ABI.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityLeftLevel {
    /// Canonical lowercase UUID of the entity that left the level.
    pub entity_id: String,
    /// Canonical registry resource location of the entity type.
    pub entity_type_id: String,
    /// Canonical resource location of the specific level the entity left.
    pub dimension_id: String,
    /// Entity X coordinate at the leave event, finite as an IEEE-754 `f64`.
    pub x: f64,
    /// Entity Y coordinate at the leave event, finite as an IEEE-754 `f64`.
    pub y: f64,
    /// Entity Z coordinate at the leave event, finite as an IEEE-754 `f64`.
    pub z: f64,
}

impl StableEncode for EntityLeftLevel {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        validate(self)?;
        encoder.write_i32(1)?;
        encode_bounded_string(&self.entity_id, encoder)?;
        encode_bounded_string(&self.entity_type_id, encoder)?;
        encode_bounded_string(&self.dimension_id, encoder)?;
        encoder.write_f64(self.x)?;
        encoder.write_f64(self.y)?;
        encoder.write_f64(self.z)
    }
}

impl StableDecode for EntityLeftLevel {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        let version = decoder.read_i32()?;
        if version != 1 {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                format!("entity.left_level schema version {version} is unsupported"),
            ));
        }
        let event = Self {
            entity_id: decode_bounded_string(decoder)?,
            entity_type_id: decode_bounded_string(decoder)?,
            dimension_id: decode_bounded_string(decoder)?,
            x: decoder.read_f64()?,
            y: decoder.read_f64()?,
            z: decoder.read_f64()?,
        };
        validate(&event)?;
        Ok(event)
    }
}

fn validate(event: &EntityLeftLevel) -> SjmlResult<()> {
    if !canonical_uuid(&event.entity_id) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity.left_level entity UUID must be canonical lowercase text",
        ));
    }
    if !canonical_resource_location(&event.entity_type_id)
        || !canonical_resource_location(&event.dimension_id)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity.left_level type and dimension IDs must be canonical resource locations",
        ));
    }
    if event.entity_id.len() > MAX_ENTITY_LEFT_LEVEL_STRING_BYTES
        || event.entity_type_id.len() > MAX_ENTITY_LEFT_LEVEL_STRING_BYTES
        || event.dimension_id.len() > MAX_ENTITY_LEFT_LEVEL_STRING_BYTES
    {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "entity.left_level string exceeds the 256-byte limit",
        ));
    }
    if !event.x.is_finite() || !event.y.is_finite() || !event.z.is_finite() {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity.left_level position values must be finite",
        ));
    }
    Ok(())
}

fn canonical_uuid(value: &str) -> bool {
    value.len() == 36
        && value.bytes().enumerate().all(|(index, byte)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                byte == b'-'
            } else {
                byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)
            }
        })
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

fn encode_bounded_string(value: &str, encoder: &mut Encoder) -> SjmlResult<()> {
    if value.len() > MAX_ENTITY_LEFT_LEVEL_STRING_BYTES {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "entity.left_level string exceeds the 256-byte limit",
        ));
    }
    encoder.write_length(value.len())?;
    encoder.write_raw(value.as_bytes())
}

fn decode_bounded_string(decoder: &mut Decoder<'_>) -> SjmlResult<String> {
    let length = decoder.read_length(MAX_ENTITY_LEFT_LEVEL_STRING_BYTES)?;
    String::from_utf8(decoder.read_raw(length)?.to_vec()).map_err(|_| {
        SjmlError::new(
            ErrorCode::InvalidUtf8,
            "entity.left_level string contains malformed UTF-8",
        )
    })
}
