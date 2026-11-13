//! Detached payload and response for cancellable logical-server entity teleports.
//!
//! Register an `entity.teleport_attempt` callback with `events.subscribe` and
//! `events.cancel`. The callback is synchronous because NeoForge applies the
//! teleport immediately after the event returns.

use crate::{Decoder, Encoder, ErrorCode, SjmlError, SjmlResult, StableDecode, StableEncode};

/// Cancellation grant required to veto server entity teleport attempts; `events.subscribe` is also required.
pub const ENTITY_TELEPORT_ATTEMPT_CANCEL_CAPABILITY: &str = "events.cancel";
/// Maximum UTF-8 byte length for a dimension or entity type resource ID.
pub const MAX_ENTITY_TELEPORT_ATTEMPT_ID_BYTES: usize = 256;
/// Maximum world coordinate accepted in one source or destination position.
pub const MAX_ENTITY_TELEPORT_ATTEMPT_COORDINATE: f64 = 30_000_000.0;

/// Stable reason categories reported by NeoForge's entity teleport event family.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EntityTeleportCause {
    /// A server command teleported an entity.
    Command,
    /// The `/spreadplayers` command teleported an entity.
    SpreadPlayersCommand,
    /// An Enderman or Shulker attempted its random teleport.
    EnderEntity,
    /// A thrown Ender Pearl attempted to teleport its player.
    EnderPearl,
    /// Chorus Fruit attempted to teleport a living entity.
    ChorusFruit,
    /// Another NeoForge or mod-defined teleport event subclass fired.
    Other,
}

impl EntityTeleportCause {
    fn wire_code(self) -> u32 {
        match self {
            Self::Command => 0,
            Self::SpreadPlayersCommand => 1,
            Self::EnderEntity => 2,
            Self::EnderPearl => 3,
            Self::ChorusFruit => 4,
            Self::Other => 5,
        }
    }

    fn from_wire_code(code: u32) -> SjmlResult<Self> {
        match code {
            0 => Ok(Self::Command),
            1 => Ok(Self::SpreadPlayersCommand),
            2 => Ok(Self::EnderEntity),
            3 => Ok(Self::EnderPearl),
            4 => Ok(Self::ChorusFruit),
            5 => Ok(Self::Other),
            value => Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                format!("entity teleport cause code {value} is not supported"),
            )),
        }
    }
}

/// Synchronous cancellable snapshot captured before a live server entity teleports.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityTeleportAttempt {
    /// Canonical UUID of the entity that is about to teleport.
    pub entity_id: String,
    /// Registry ID of the entity type.
    pub entity_type_id: String,
    /// Resource ID of the entity's current dimension. These teleport hooks do not change dimension.
    pub dimension_id: String,
    /// NeoForge teleport operation that produced this attempt.
    pub cause: EntityTeleportCause,
    /// Current world-space X coordinate before teleportation.
    pub from_x: f64,
    /// Current world-space Y coordinate before teleportation.
    pub from_y: f64,
    /// Current world-space Z coordinate before teleportation.
    pub from_z: f64,
    /// Destination world-space X coordinate.
    pub target_x: f64,
    /// Destination world-space Y coordinate.
    pub target_y: f64,
    /// Destination world-space Z coordinate.
    pub target_z: f64,
}

impl StableEncode for EntityTeleportAttempt {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        validate_attempt(self)?;
        encoder.write_i32(1)?;
        encode_bounded_string(&self.entity_id, encoder)?;
        encode_bounded_string(&self.entity_type_id, encoder)?;
        encode_bounded_string(&self.dimension_id, encoder)?;
        encoder.write_u32(self.cause.wire_code())?;
        encoder.write_f64(self.from_x)?;
        encoder.write_f64(self.from_y)?;
        encoder.write_f64(self.from_z)?;
        encoder.write_f64(self.target_x)?;
        encoder.write_f64(self.target_y)?;
        encoder.write_f64(self.target_z)
    }
}

impl StableDecode for EntityTeleportAttempt {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        let schema = decoder.read_i32()?;
        if schema != 1 {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                format!("EntityTeleportAttempt schema version {schema} is unsupported"),
            ));
        }
        let attempt = Self {
            entity_id: decode_bounded_string(decoder)?,
            entity_type_id: decode_bounded_string(decoder)?,
            dimension_id: decode_bounded_string(decoder)?,
            cause: EntityTeleportCause::from_wire_code(decoder.read_u32()?)?,
            from_x: decoder.read_f64()?,
            from_y: decoder.read_f64()?,
            from_z: decoder.read_f64()?,
            target_x: decoder.read_f64()?,
            target_y: decoder.read_f64()?,
            target_z: decoder.read_f64()?,
        };
        validate_attempt(&attempt)?;
        Ok(attempt)
    }
}

/// Synchronous response returned by an `entity.teleport_attempt` callback.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EntityTeleportAttemptAction {
    /// Allow the original NeoForge teleport to continue.
    Pass,
    /// Cancel this teleport before Minecraft applies it.
    Cancel,
}

impl StableEncode for EntityTeleportAttemptAction {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        encoder.write_u8(match self {
            Self::Pass => 0,
            Self::Cancel => 1,
        })
    }
}

impl StableDecode for EntityTeleportAttemptAction {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        match decoder.read_u8()? {
            0 => Ok(Self::Pass),
            1 => Ok(Self::Cancel),
            value => Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                format!("entity teleport action code {value} is not supported"),
            )),
        }
    }
}

fn validate_attempt(event: &EntityTeleportAttempt) -> SjmlResult<()> {
    if !canonical_uuid(&event.entity_id)
        || !canonical_resource_location(&event.entity_type_id)
        || !canonical_resource_location(&event.dimension_id)
        || !valid_position(event.from_x, event.from_y, event.from_z)
        || !valid_position(event.target_x, event.target_y, event.target_z)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity teleport attempt contains an invalid UUID, resource ID, or coordinate",
        ));
    }
    Ok(())
}

fn encode_bounded_string(value: &str, encoder: &mut Encoder) -> SjmlResult<()> {
    if value.len() > MAX_ENTITY_TELEPORT_ATTEMPT_ID_BYTES {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "entity teleport attempt resource ID exceeds the 256-byte limit",
        ));
    }
    encoder.write_length(value.len())?;
    encoder.write_raw(value.as_bytes())
}

fn decode_bounded_string(decoder: &mut Decoder<'_>) -> SjmlResult<String> {
    let length = decoder.read_length(MAX_ENTITY_TELEPORT_ATTEMPT_ID_BYTES)?;
    String::from_utf8(decoder.read_raw(length)?.to_vec()).map_err(|_| {
        SjmlError::new(
            ErrorCode::InvalidUtf8,
            "entity teleport attempt resource ID is not valid UTF-8",
        )
    })
}

fn valid_position(x: f64, y: f64, z: f64) -> bool {
    [x, y, z]
        .into_iter()
        .all(|value| value.is_finite() && value.abs() <= MAX_ENTITY_TELEPORT_ATTEMPT_COORDINATE)
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
    use super::*;
    use crate::{from_bytes, to_bytes};

    fn event() -> EntityTeleportAttempt {
        EntityTeleportAttempt {
            entity_id: "68ccfd1a-588d-4eb4-a0c9-31046942d74f".into(),
            entity_type_id: "minecraft:pig".into(),
            dimension_id: "minecraft:overworld".into(),
            cause: EntityTeleportCause::EnderPearl,
            from_x: 1.25,
            from_y: 64.0,
            from_z: -2.5,
            target_x: 120.0,
            target_y: 70.0,
            target_z: 121.0,
        }
    }

    #[test]
    fn event_round_trips_the_java_schema_and_cause_codes() {
        let event = event();
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(&encoded[..4], &1_i32.to_le_bytes());
        assert_eq!(
            from_bytes::<EntityTeleportAttempt>(&encoded).unwrap(),
            event
        );
        assert_eq!(
            from_bytes::<EntityTeleportAttemptAction>(
                &to_bytes(&EntityTeleportAttemptAction::Cancel).unwrap()
            )
            .unwrap(),
            EntityTeleportAttemptAction::Cancel
        );
    }

    #[test]
    fn invalid_identity_and_coordinates_are_rejected() {
        let mut invalid_uuid = event();
        invalid_uuid.entity_id = "not-a-uuid".into();
        assert_eq!(
            to_bytes(&invalid_uuid).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );

        let mut invalid_dimension = event();
        invalid_dimension.dimension_id = "Minecraft:overworld".into();
        assert_eq!(
            to_bytes(&invalid_dimension).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );

        let mut non_finite = event();
        non_finite.target_y = f64::NAN;
        assert_eq!(
            to_bytes(&non_finite).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );

        let mut out_of_range = event();
        out_of_range.target_x = MAX_ENTITY_TELEPORT_ATTEMPT_COORDINATE + 1.0;
        assert_eq!(
            to_bytes(&out_of_range).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn response_rejects_unknown_action_values() {
        assert_eq!(
            from_bytes::<EntityTeleportAttemptAction>(&[0]).unwrap(),
            EntityTeleportAttemptAction::Pass
        );
        assert_eq!(
            from_bytes::<EntityTeleportAttemptAction>(&[1]).unwrap(),
            EntityTeleportAttemptAction::Cancel
        );
        assert_eq!(
            from_bytes::<EntityTeleportAttemptAction>(&[2])
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }
}
