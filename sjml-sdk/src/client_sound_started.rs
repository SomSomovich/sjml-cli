//! Observation-only snapshots for sounds that began playback on the physical client.
//!
//! Subscribe with `events::subscribe("client.sound_started", EventPriority::Normal)` and a typed
//! `#[event(name = "client.sound_started")]` callback. The module must have both
//! `events.subscribe` and `client.audio.observe`. The callback receives only a canonical sound
//! ID, a vanilla category, finite bounded levels, and detached world coordinates; it cannot
//! affect playback or retain a Minecraft sound object. The host invokes at most
//! [`MAX_CALLBACKS_PER_TICK`] callbacks per client tick across all observed sounds.

use crate::serialization::{Decoder, Encoder};
use crate::{ErrorCode, SjmlError, SjmlResult, StableDecode, StableEncode};

/// Event name used with [`crate::events::subscribe`].
pub const EVENT_NAME: &str = "client.sound_started";
/// Separate grant required to observe locally started sound sources.
pub const OBSERVE_CAPABILITY: &str = "client.audio.observe";
/// Maximum UTF-8 byte length of the detached sound resource ID.
pub const MAX_SOUND_ID_BYTES: usize = 256;
/// Maximum volume represented by a sound-start notification.
pub const MAX_VOLUME: f32 = 64.0;
/// Maximum pitch represented by a sound-start notification.
pub const MAX_PITCH: f32 = 16.0;
/// Maximum absolute value of one position coordinate.
pub const MAX_POSITION: f64 = 30_000_000.0;
/// Maximum guest callbacks from all sound-start notifications in one client tick.
pub const MAX_CALLBACKS_PER_TICK: usize = 64;
/// Maximum encoded bytes for one detached sound-start event.
pub const MAX_PAYLOAD_BYTES: usize = 4 + 4 + MAX_SOUND_ID_BYTES + 4 + 4 + 4 + 8 * 3;

/// One vanilla sound control category, encoded with the stable codes from Minecraft 1.21.1.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientSoundCategory {
    /// Master volume.
    Master,
    /// Music volume.
    Music,
    /// Record and jukebox volume.
    Records,
    /// Weather volume.
    Weather,
    /// Block sound volume.
    Blocks,
    /// Hostile mob volume.
    Hostile,
    /// Neutral mob volume.
    Neutral,
    /// Player volume.
    Players,
    /// Ambient volume.
    Ambient,
    /// Voice volume.
    Voice,
}

impl ClientSoundCategory {
    fn wire_code(self) -> u32 {
        match self {
            Self::Master => 0,
            Self::Music => 1,
            Self::Records => 2,
            Self::Weather => 3,
            Self::Blocks => 4,
            Self::Hostile => 5,
            Self::Neutral => 6,
            Self::Players => 7,
            Self::Ambient => 8,
            Self::Voice => 9,
        }
    }

    fn from_wire(value: u32) -> SjmlResult<Self> {
        match value {
            0 => Ok(Self::Master),
            1 => Ok(Self::Music),
            2 => Ok(Self::Records),
            3 => Ok(Self::Weather),
            4 => Ok(Self::Blocks),
            5 => Ok(Self::Hostile),
            6 => Ok(Self::Neutral),
            7 => Ok(Self::Players),
            8 => Ok(Self::Ambient),
            9 => Ok(Self::Voice),
            _ => Err(invalid("sound category code is unsupported")),
        }
    }
}

/// Detached sound-start notification. It contains no `SoundInstance` or audio-channel handle.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientSoundStarted {
    /// Canonical lowercase sound event ID such as `minecraft:entity.player.levelup`.
    pub sound_id: String,
    /// Vanilla volume control associated with the sound.
    pub category: ClientSoundCategory,
    /// Effective source volume, finite and in `0..=64`.
    pub volume: f32,
    /// Effective source pitch, finite and in `0..=16`.
    pub pitch: f32,
    /// Sound source X position, finite and in `-30_000_000..=30_000_000`.
    pub x: f64,
    /// Sound source Y position, finite and in `-30_000_000..=30_000_000`.
    pub y: f64,
    /// Sound source Z position, finite and in `-30_000_000..=30_000_000`.
    pub z: f64,
}

impl StableEncode for ClientSoundStarted {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        validate(self)?;
        encoder.write_u32(1)?;
        encoder.write_length(self.sound_id.len())?;
        encoder.write_raw(self.sound_id.as_bytes())?;
        encoder.write_u32(self.category.wire_code())?;
        encoder.write_f32(self.volume)?;
        encoder.write_f32(self.pitch)?;
        encoder.write_f64(self.x)?;
        encoder.write_f64(self.y)?;
        encoder.write_f64(self.z)
    }
}

impl StableDecode for ClientSoundStarted {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        if decoder.remaining() > MAX_PAYLOAD_BYTES {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "client.sound_started payload exceeds the 300-byte limit",
            ));
        }
        let version = decoder.read_u32()?;
        if version != 1 {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                format!("ClientSoundStarted schema version {version} is unsupported"),
            ));
        }
        let sound_id_length = decoder.read_length(MAX_SOUND_ID_BYTES)?;
        let sound_id =
            String::from_utf8(decoder.read_raw(sound_id_length)?.to_vec()).map_err(|_| {
                SjmlError::new(ErrorCode::InvalidUtf8, "client sound ID is not valid UTF-8")
            })?;
        let event = Self {
            sound_id,
            category: ClientSoundCategory::from_wire(decoder.read_u32()?)?,
            volume: decoder.read_f32()?,
            pitch: decoder.read_f32()?,
            x: decoder.read_f64()?,
            y: decoder.read_f64()?,
            z: decoder.read_f64()?,
        };
        validate(&event)?;
        Ok(event)
    }
}

fn validate(event: &ClientSoundStarted) -> SjmlResult<()> {
    if event.sound_id.len() > MAX_SOUND_ID_BYTES {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "client sound ID exceeds the 256-byte limit",
        ));
    }
    if !canonical_resource_location(&event.sound_id) {
        return Err(invalid(
            "client sound ID must be a canonical lowercase resource location",
        ));
    }
    if !event.volume.is_finite()
        || !(0.0..=MAX_VOLUME).contains(&event.volume)
        || !event.pitch.is_finite()
        || !(0.0..=MAX_PITCH).contains(&event.pitch)
    {
        return Err(invalid(
            "client sound volume or pitch is outside its finite bound",
        ));
    }
    if !event.x.is_finite()
        || !event.y.is_finite()
        || !event.z.is_finite()
        || event.x.abs() > MAX_POSITION
        || event.y.abs() > MAX_POSITION
        || event.z.abs() > MAX_POSITION
    {
        return Err(invalid(
            "client sound position is outside its finite world-coordinate bound",
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
        && namespace.bytes().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'_' | b'.' | b'-')
        })
        && path.bytes().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'_' | b'.' | b'-' | b'/')
        })
}

fn invalid(message: &str) -> SjmlError {
    SjmlError::new(ErrorCode::InvalidArgument, message)
}

#[cfg(test)]
mod tests {
    use super::{
        ClientSoundCategory, ClientSoundStarted, MAX_CALLBACKS_PER_TICK, MAX_PAYLOAD_BYTES,
        MAX_POSITION, MAX_SOUND_ID_BYTES,
    };
    use crate::serialization::{from_bytes, to_bytes};
    use crate::ErrorCode;

    fn event() -> ClientSoundStarted {
        ClientSoundStarted {
            sound_id: "minecraft:entity.player.levelup".into(),
            category: ClientSoundCategory::Players,
            volume: 1.25,
            pitch: 0.8,
            x: -4.5,
            y: 72.0,
            z: 19.25,
        }
    }

    #[test]
    fn stable_round_trip_preserves_detached_sound_data() {
        let original = event();
        let bytes = to_bytes(&original).unwrap();
        assert!(bytes.len() <= MAX_PAYLOAD_BYTES);
        assert_eq!(u32::from_le_bytes(bytes[0..4].try_into().unwrap()), 1);
        let category_offset = 4 + 4 + original.sound_id.len();
        assert_eq!(
            u32::from_le_bytes(
                bytes[category_offset..category_offset + 4]
                    .try_into()
                    .unwrap()
            ),
            7
        );
        assert_eq!(from_bytes::<ClientSoundStarted>(&bytes).unwrap(), original);
        assert_eq!(MAX_CALLBACKS_PER_TICK, 64);
    }

    #[test]
    fn codec_rejects_bad_category_schema_and_trailing_bytes() {
        let bytes = to_bytes(&event()).unwrap();
        let mut bad_category = bytes.clone();
        let category_offset = 4 + 4 + event().sound_id.len();
        bad_category[category_offset..category_offset + 4].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(
            from_bytes::<ClientSoundStarted>(&bad_category)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );

        let mut bad_schema = bytes.clone();
        bad_schema[0..4].copy_from_slice(&2_u32.to_le_bytes());
        assert_eq!(
            from_bytes::<ClientSoundStarted>(&bad_schema)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );

        let mut trailing = bytes;
        trailing.push(0);
        assert!(from_bytes::<ClientSoundStarted>(&trailing).is_err());
    }

    #[test]
    fn codec_rejects_noncanonical_id_nonfinite_levels_and_out_of_range_position() {
        for invalid in [
            ClientSoundStarted {
                sound_id: "Minecraft:Foo".into(),
                ..event()
            },
            ClientSoundStarted {
                volume: f32::NAN,
                ..event()
            },
            ClientSoundStarted {
                pitch: f32::INFINITY,
                ..event()
            },
            ClientSoundStarted {
                x: MAX_POSITION + 1.0,
                ..event()
            },
            ClientSoundStarted {
                z: f64::NEG_INFINITY,
                ..event()
            },
            ClientSoundStarted {
                sound_id: "x".repeat(MAX_SOUND_ID_BYTES + 1),
                ..event()
            },
        ] {
            assert_eq!(
                to_bytes(&invalid).unwrap_err().code(),
                if invalid.sound_id.len() > MAX_SOUND_ID_BYTES {
                    ErrorCode::PayloadTooLarge
                } else {
                    ErrorCode::InvalidArgument
                }
            );
        }
    }
}
