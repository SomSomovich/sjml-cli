//! Synchronous shield-block veto values for `entity.shield_block_attempt`.
//!
//! The host emits this callback only when vanilla considers a living entity to be blocking
//! an incoming hit with a shield. Returning [`EntityShieldBlockAttemptAction::Cancel`] makes
//! NeoForge treat the shield as ineligible, so the incoming damage proceeds normally. The
//! callback cannot force a block or change either the blocked amount or shield durability.

use crate::{Decoder, Encoder, ErrorCode, SjmlError, SjmlResult, StableDecode, StableEncode};

/// Maximum UTF-8 byte length of one detached registry ID in a shield-block callback.
pub const MAX_ENTITY_SHIELD_BLOCK_ID_BYTES: usize = 256;
/// Maximum damage amount represented in a shield-block callback.
pub const MAX_ENTITY_SHIELD_BLOCK_DAMAGE: f32 = 1_000_000.0;

/// Detached logical-server values observed when vanilla is about to apply a shield block.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityShieldBlockAttempt {
    /// Canonical UUID string of the living entity that would block the hit.
    pub entity_id: String,
    /// Entity type resource location, such as `minecraft:player`.
    pub entity_type_id: String,
    /// Dimension resource location where damage occurred.
    pub dimension_id: String,
    /// Damage-type resource location.
    pub damage_type_id: String,
    /// UUID of the causing entity, when the source has one.
    pub causing_entity_id: Option<String>,
    /// UUID of the direct source entity, such as a projectile, when present.
    pub direct_entity_id: Option<String>,
    /// Whether vanilla's original shield-facing and eligibility checks passed.
    pub original_block: bool,
    /// Whether this event is still marked as blocking when this listener captures it.
    pub blocked: bool,
    /// Incoming amount vanilla originally offered to the shield.
    pub original_blocked_damage: f32,
    /// Amount currently assigned to shield blocking, before guest callbacks run.
    pub blocked_damage: f32,
    /// Shield durability damage currently assigned by NeoForge, before guest callbacks run.
    pub shield_damage: f32,
}

impl StableEncode for EntityShieldBlockAttempt {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        validate(self)?;
        encoder.write_i32(1)?;
        encode_string(&self.entity_id, encoder)?;
        encode_string(&self.entity_type_id, encoder)?;
        encode_string(&self.dimension_id, encoder)?;
        encode_string(&self.damage_type_id, encoder)?;
        encode_optional_string(self.causing_entity_id.as_deref(), encoder)?;
        encode_optional_string(self.direct_entity_id.as_deref(), encoder)?;
        encoder.write_u8(u8::from(self.original_block))?;
        encoder.write_u8(u8::from(self.blocked))?;
        encoder.write_f32(self.original_blocked_damage)?;
        encoder.write_f32(self.blocked_damage)?;
        encoder.write_f32(self.shield_damage)
    }
}

impl StableDecode for EntityShieldBlockAttempt {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        let version = decoder.read_i32()?;
        if version != 1 {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                format!("EntityShieldBlockAttempt schema version {version} is unsupported"),
            ));
        }
        let event = Self {
            entity_id: decode_string(decoder)?,
            entity_type_id: decode_string(decoder)?,
            dimension_id: decode_string(decoder)?,
            damage_type_id: decode_string(decoder)?,
            causing_entity_id: decode_optional_string(decoder)?,
            direct_entity_id: decode_optional_string(decoder)?,
            original_block: decode_bool(decoder)?,
            blocked: decode_bool(decoder)?,
            original_blocked_damage: decoder.read_f32()?,
            blocked_damage: decoder.read_f32()?,
            shield_damage: decoder.read_f32()?,
        };
        validate(&event)?;
        Ok(event)
    }
}

/// Synchronous pass/cancel result for `entity.shield_block_attempt`.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EntityShieldBlockAttemptAction {
    /// Keep the shield block decision made by Minecraft and other event listeners.
    Pass,
    /// Treat the shield as ineligible for this hit; damage proceeds through vanilla handling.
    Cancel,
}

impl StableEncode for EntityShieldBlockAttemptAction {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        encoder.write_u8(match self {
            Self::Pass => 0,
            Self::Cancel => 1,
        })
    }
}

impl StableDecode for EntityShieldBlockAttemptAction {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        match decoder.read_u8()? {
            0 => Ok(Self::Pass),
            1 => Ok(Self::Cancel),
            code => Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                format!("entity shield-block action code {code} is not supported"),
            )),
        }
    }
}

fn validate(event: &EntityShieldBlockAttempt) -> SjmlResult<()> {
    let valid_amount = |amount: f32| {
        amount.is_finite() && (0.0..=MAX_ENTITY_SHIELD_BLOCK_DAMAGE).contains(&amount)
    };
    if !canonical_uuid(&event.entity_id)
        || event
            .causing_entity_id
            .as_deref()
            .is_some_and(|value| !canonical_uuid(value))
        || event
            .direct_entity_id
            .as_deref()
            .is_some_and(|value| !canonical_uuid(value))
        || !canonical_resource_location(&event.entity_type_id)
        || !canonical_resource_location(&event.dimension_id)
        || !canonical_resource_location(&event.damage_type_id)
        || !valid_amount(event.original_blocked_damage)
        || !valid_amount(event.blocked_damage)
        || !valid_amount(event.shield_damage)
        || event.blocked_damage > event.original_blocked_damage
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity shield-block attempt contains invalid detached identity or damage values",
        ));
    }
    Ok(())
}

fn encode_string(value: &str, encoder: &mut Encoder) -> SjmlResult<()> {
    if value.is_empty() || value.len() > MAX_ENTITY_SHIELD_BLOCK_ID_BYTES {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "entity shield-block ID must contain 1 to 256 UTF-8 bytes",
        ));
    }
    encoder.write_length(value.len())?;
    encoder.write_raw(value.as_bytes())
}

fn decode_string(decoder: &mut Decoder<'_>) -> SjmlResult<String> {
    let length = decoder.read_length(MAX_ENTITY_SHIELD_BLOCK_ID_BYTES)?;
    let value = String::from_utf8(decoder.read_raw(length)?.to_vec()).map_err(|_| {
        SjmlError::new(
            ErrorCode::InvalidUtf8,
            "entity shield-block ID is not valid UTF-8",
        )
    })?;
    if value.is_empty() {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity shield-block ID must not be empty",
        ));
    }
    Ok(value)
}

fn encode_optional_string(value: Option<&str>, encoder: &mut Encoder) -> SjmlResult<()> {
    match value {
        Some(value) => {
            encoder.write_u8(1)?;
            encode_string(value, encoder)
        }
        None => encoder.write_u8(0),
    }
}

fn decode_optional_string(decoder: &mut Decoder<'_>) -> SjmlResult<Option<String>> {
    match decoder.read_u8()? {
        0 => Ok(None),
        1 => Ok(Some(decode_string(decoder)?)),
        _ => Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity shield-block optional entity marker must be zero or one",
        )),
    }
}

fn decode_bool(decoder: &mut Decoder<'_>) -> SjmlResult<bool> {
    match decoder.read_u8()? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity shield-block boolean must be zero or one",
        )),
    }
}

fn canonical_uuid(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 36
        && bytes.iter().enumerate().all(|(index, byte)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                *byte == b'-'
            } else {
                byte.is_ascii_digit() || (b'a'..=b'f').contains(byte)
            }
        })
}

fn canonical_resource_location(value: &str) -> bool {
    let Some((namespace, path)) = value.split_once(':') else {
        return false;
    };
    !namespace.is_empty()
        && !path.is_empty()
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
    use super::{EntityShieldBlockAttempt, EntityShieldBlockAttemptAction};
    use crate::{from_bytes, to_bytes};

    fn sample() -> EntityShieldBlockAttempt {
        EntityShieldBlockAttempt {
            entity_id: "00000000-0000-0000-0000-000000000001".into(),
            entity_type_id: "minecraft:player".into(),
            dimension_id: "minecraft:overworld".into(),
            damage_type_id: "minecraft:arrow".into(),
            causing_entity_id: Some("00000000-0000-0000-0000-000000000002".into()),
            direct_entity_id: Some("00000000-0000-0000-0000-000000000003".into()),
            original_block: true,
            blocked: true,
            original_blocked_damage: 7.0,
            blocked_damage: 7.0,
            shield_damage: 7.0,
        }
    }

    #[test]
    fn shield_block_values_round_trip_with_optional_source_entities() {
        let event = sample();
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(
            from_bytes::<EntityShieldBlockAttempt>(&encoded).unwrap(),
            event
        );

        let mut no_entity_source = sample();
        no_entity_source.causing_entity_id = None;
        no_entity_source.direct_entity_id = None;
        let encoded = to_bytes(&no_entity_source).unwrap();
        assert_eq!(
            from_bytes::<EntityShieldBlockAttempt>(&encoded).unwrap(),
            no_entity_source
        );
    }

    #[test]
    fn shield_block_action_has_stable_veto_codes() {
        for action in [
            EntityShieldBlockAttemptAction::Pass,
            EntityShieldBlockAttemptAction::Cancel,
        ] {
            let encoded = to_bytes(&action).unwrap();
            assert_eq!(
                from_bytes::<EntityShieldBlockAttemptAction>(&encoded).unwrap(),
                action
            );
        }
        assert!(from_bytes::<EntityShieldBlockAttemptAction>(&[2]).is_err());
    }

    #[test]
    fn invalid_damage_values_or_resource_ids_are_rejected() {
        let mut event = sample();
        event.shield_damage = f32::NAN;
        assert!(to_bytes(&event).is_err());

        let mut event = sample();
        event.blocked_damage = 7.5;
        assert!(to_bytes(&event).is_err());

        let mut event = sample();
        event.dimension_id = "Minecraft:overworld".into();
        assert!(to_bytes(&event).is_err());
    }
}
