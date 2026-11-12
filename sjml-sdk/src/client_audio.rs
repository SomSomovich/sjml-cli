//! Bounded client-local sound playback through Minecraft's sound manager.
//!
//! One-shot playback requires `client.audio.play`; looping playback requires `client.audio.loop`.
//! Requests run on the physical client main thread, accept only a sound event available in the
//! active resource set, and never send sound playback over the network. The host rate-limits
//! starts per module and stops only looping instances owned by the calling module.

use crate::{ErrorCode, SjmlError, SjmlResult};

/// Maximum UTF-8 byte length of a Minecraft sound resource ID.
pub const MAX_SOUND_ID_BYTES: usize = 256;
/// Largest volume accepted by [`play`].
pub const MAX_SOUND_VOLUME: f32 = 4.0;
/// Smallest pitch accepted by [`play`].
pub const MIN_SOUND_PITCH: f32 = 0.5;
/// Largest pitch accepted by [`play`].
pub const MAX_SOUND_PITCH: f32 = 2.0;
/// Maximum active looping sounds a module may own at one time.
pub const MAX_ACTIVE_LOOPING_SOUNDS_PER_MODULE: usize = 8;

/// One of Minecraft's built-in sound categories.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum SoundCategory {
    Master = 0,
    Music = 1,
    Records = 2,
    Weather = 3,
    Blocks = 4,
    Hostile = 5,
    Neutral = 6,
    Players = 7,
    Ambient = 8,
    Voice = 9,
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_client_audio_v1")]
extern "C" {
    #[link_name = "play"]
    fn imported_play(
        sound_id_pointer: i32,
        sound_id_length: i32,
        category: i32,
        volume_bits: i32,
        pitch_bits: i32,
    ) -> i32;

    #[link_name = "start_loop"]
    fn imported_start_loop(
        sound_id_pointer: i32,
        sound_id_length: i32,
        category: i32,
        volume_bits: i32,
        pitch_bits: i32,
    ) -> i64;

    #[link_name = "stop_loop"]
    fn imported_stop_loop(handle: i32) -> i32;
}

/// Opaque loop handle scoped to the SJML module runtime that created it.
#[must_use = "store the handle to stop this loop before the module runtime closes"]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct LoopingSoundHandle(u32);

impl LoopingSoundHandle {
    /// Returns the opaque numeric value for storage or comparison.
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Plays a sound event locally at the client's listener position.
///
/// `sound_id` is a canonical lowercase Minecraft resource ID such as
/// `minecraft:entity.player.levelup`; it must have an active sound resource. Volume is finite
/// in `0..=4` and pitch is finite in `0.5..=2`. Categories map to Minecraft's MASTER, MUSIC,
/// RECORDS, WEATHER, BLOCKS, HOSTILE, NEUTRAL, PLAYERS, AMBIENT, and VOICE controls. Each module
/// may make at most five requests in any two-second window. Server and static-registration
/// contexts do not receive a usable import.
pub fn play(sound_id: &str, category: SoundCategory, volume: f32, pitch: f32) -> SjmlResult<()> {
    validate_sound_id(sound_id)?;
    validate_levels(volume, pitch)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the borrowed resource ID stays live during this synchronous import. The host
        // copies and validates the bounded UTF-8 bytes before returning.
        let status = unsafe {
            imported_play(
                sound_id.as_ptr() as usize as u32 as i32,
                sound_id.len() as i32,
                category as u32 as i32,
                volume.to_bits() as i32,
                pitch.to_bits() as i32,
            )
        };
        if status == 0 {
            Ok(())
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected the client sound request",
            ))
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = category;
        Err(SjmlError::new(
            ErrorCode::UnknownErrorCode,
            "client audio is available only to wasm32 guests",
        ))
    }
}

/// Starts a local looping sound and returns its module-owned handle.
///
/// The module must request and receive `client.audio.loop`; the host allows at most eight active
/// loops per module and at most five playback starts per two-second window. Handles are valid only
/// for the creating module runtime. Stopping a loop cannot stop another module's sound, and all
/// remaining handles are stopped automatically when their owner runtime is deactivated.
pub fn start_loop(
    sound_id: &str,
    category: SoundCategory,
    volume: f32,
    pitch: f32,
) -> SjmlResult<LoopingSoundHandle> {
    validate_sound_id(sound_id)?;
    validate_levels(volume, pitch)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the borrowed resource ID stays live during this synchronous import. The host
        // copies and validates the bounded UTF-8 bytes before returning.
        let packed = unsafe {
            imported_start_loop(
                sound_id.as_ptr() as usize as u32 as i32,
                sound_id.len() as i32,
                category as u32 as i32,
                volume.to_bits() as i32,
                pitch.to_bits() as i32,
            ) as u64
        };
        let status = (packed >> 32) as u32;
        if status != 0 {
            return Err(SjmlError::from_abi_code(
                status,
                "host rejected the looping client sound request",
            ));
        }
        let handle = packed as u32;
        if handle == 0 || handle > i32::MAX as u32 {
            return Err(invalid("host returned an invalid looping sound handle"));
        }
        Ok(LoopingSoundHandle(handle))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = category;
        Err(SjmlError::new(
            ErrorCode::UnknownErrorCode,
            "client audio is available only to wasm32 guests",
        ))
    }
}

/// Stops one looping sound owned by the current module runtime.
pub fn stop_loop(handle: LoopingSoundHandle) -> SjmlResult<()> {
    if handle.0 == 0 || handle.0 > i32::MAX as u32 {
        return Err(invalid("looping sound handle is invalid"));
    }
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host verifies this opaque handle against the calling module's own registry.
        let status = unsafe { imported_stop_loop(handle.0 as i32) };
        if status == 0 {
            Ok(())
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected the looping sound handle",
            ))
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(SjmlError::new(
            ErrorCode::UnknownErrorCode,
            "client audio is available only to wasm32 guests",
        ))
    }
}

fn validate_sound_id(sound_id: &str) -> SjmlResult<()> {
    if sound_id.len() < 3 || sound_id.len() > MAX_SOUND_ID_BYTES {
        return Err(invalid("sound ID must be 3–256 UTF-8 bytes"));
    }
    let Some((namespace, path)) = sound_id.split_once(':') else {
        return Err(invalid(
            "sound ID must use the canonical namespace:path form",
        ));
    };
    if namespace.is_empty()
        || path.is_empty()
        || !namespace.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_.-".contains(&byte)
        })
        || !path.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_./-".contains(&byte)
        })
        || path.starts_with('/')
        || path.ends_with('/')
        || path.contains("//")
        || path
            .split('/')
            .any(|segment| segment == "." || segment == "..")
    {
        return Err(invalid(
            "sound ID must be a canonical lowercase namespace:path ID",
        ));
    }
    Ok(())
}

fn validate_levels(volume: f32, pitch: f32) -> SjmlResult<()> {
    if !volume.is_finite()
        || !(0.0..=MAX_SOUND_VOLUME).contains(&volume)
        || !pitch.is_finite()
        || !(MIN_SOUND_PITCH..=MAX_SOUND_PITCH).contains(&pitch)
    {
        return Err(invalid("sound volume must be 0–4 and pitch must be 0.5–2"));
    }
    Ok(())
}

fn invalid(message: impl Into<String>) -> SjmlError {
    SjmlError::new(ErrorCode::InvalidArgument, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_canonical_vanilla_and_mod_sound_ids() {
        assert!(validate_sound_id("minecraft:entity.player.levelup").is_ok());
        assert!(validate_sound_id("example_mod:ui/notify").is_ok());
        for invalid_id in [
            "",
            "stone",
            "Minecraft:stone",
            "example:",
            "example:/sound",
            "example:a/../b",
        ] {
            assert_eq!(
                validate_sound_id(invalid_id).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
        }
        assert_eq!(
            validate_sound_id(&"x".repeat(MAX_SOUND_ID_BYTES + 1))
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn rejects_nonfinite_or_out_of_range_volume_and_pitch() {
        assert!(validate_levels(0.0, 0.5).is_ok());
        assert!(validate_levels(4.0, 2.0).is_ok());
        for (volume, pitch) in [
            (f32::NAN, 1.0),
            (f32::INFINITY, 1.0),
            (-0.01, 1.0),
            (4.01, 1.0),
            (1.0, f32::NAN),
            (1.0, f32::INFINITY),
            (1.0, 0.49),
            (1.0, 2.01),
        ] {
            assert_eq!(
                validate_levels(volume, pitch).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
        }
    }

    #[test]
    fn category_codes_match_the_host_mapping() {
        assert_eq!(SoundCategory::Master as u32, 0);
        assert_eq!(SoundCategory::Blocks as u32, 4);
        assert_eq!(SoundCategory::Voice as u32, 9);
    }

    #[test]
    fn native_call_fails_closed_after_validating_arguments() {
        assert_eq!(
            play(
                "minecraft:entity.player.levelup",
                SoundCategory::Players,
                1.0,
                1.0
            )
            .unwrap_err()
            .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            play("bad ID", SoundCategory::Players, 1.0, 1.0)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn native_looping_calls_fail_closed_after_validating_arguments() {
        assert_eq!(
            start_loop(
                "minecraft:entity.player.levelup",
                SoundCategory::Music,
                1.0,
                1.0,
            )
            .unwrap_err()
            .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            start_loop("bad ID", SoundCategory::Music, 1.0, 1.0)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }
}
