//! Bounded client-local sound playback through Minecraft's sound manager.
//!
//! One-shot playback requires `client.audio.play`; looping playback requires `client.audio.loop`.
//! Requests run on the physical client main thread, accept only a sound event available in the
//! active resource set, and never send sound playback over the network. The host rate-limits
//! starts per module and stops only exact instances owned by the calling module.

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
/// Maximum module-owned one-shot sounds that can be stopped at one time.
pub const MAX_ACTIVE_TRACKED_SOUNDS_PER_MODULE: usize = 16;
/// Longest fade that may be scheduled for one tracked sound, in client ticks.
pub const MAX_SOUND_FADE_DURATION_TICKS: u32 = 1_200;

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

    #[link_name = "play_tracked"]
    fn imported_play_tracked(
        sound_id_pointer: i32,
        sound_id_length: i32,
        category: i32,
        volume_bits: i32,
        pitch_bits: i32,
    ) -> i64;

    #[link_name = "stop_tracked"]
    fn imported_stop_tracked(handle: i32) -> i32;

    #[link_name = "fade_tracked"]
    fn imported_fade_tracked(handle: i32, volume_bits: i32, duration_ticks: i32) -> i32;

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

/// Opaque one-shot sound handle scoped to the SJML module runtime that created it.
#[must_use = "stop the sound with this handle or let runtime cleanup stop it"]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TrackedSoundHandle(u32);

impl TrackedSoundHandle {
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

/// Plays a one-shot sound locally and returns a handle that can stop that exact playback.
///
/// The module must request and receive `client.audio.play`. The host permits at most sixteen
/// retained tracked handles per module runtime and 512 across the physical client. A sound that
/// completes naturally still owns its handle until `stop_tracked` releases it or the runtime
/// closes. Tracked playback shares the existing five-starts-per-two-seconds rate limit with
/// [`play`] and [`start_loop`]. Stopping cannot affect another module's playback.
pub fn play_tracked(
    sound_id: &str,
    category: SoundCategory,
    volume: f32,
    pitch: f32,
) -> SjmlResult<TrackedSoundHandle> {
    validate_sound_id(sound_id)?;
    validate_levels(volume, pitch)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the borrowed resource ID stays live during this synchronous import. The host
        // copies and validates the bounded UTF-8 bytes before returning.
        let packed = unsafe {
            imported_play_tracked(
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
                "host rejected the tracked client sound request",
            ));
        }
        let handle = packed as u32;
        if handle == 0 || handle > i32::MAX as u32 {
            return Err(invalid("host returned an invalid tracked sound handle"));
        }
        Ok(TrackedSoundHandle(handle))
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

/// Stops the exact one-shot sound instance owned by the current module runtime.
pub fn stop_tracked(handle: TrackedSoundHandle) -> SjmlResult<()> {
    if handle.0 == 0 || handle.0 > i32::MAX as u32 {
        return Err(invalid("tracked sound handle is invalid"));
    }
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host resolves this opaque handle only in the calling module's own registry.
        let status = unsafe { imported_stop_tracked(handle.0 as i32) };
        if status == 0 {
            Ok(())
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected the tracked sound handle",
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

/// Fades an owned tracked sound from its current volume to `target_volume` over client ticks.
///
/// Use `play_tracked(..., 0.0, pitch)` followed by this function for a fade-in. A fade-out uses
/// `target_volume = 0.0`. The target must be finite and in `0..=4`, and the duration must be in
/// `1..=1200` ticks (up to one minute). A new call replaces any fade already in progress for this
/// exact sound instance. The sound remains owned until `stop_tracked` or runtime cleanup.
pub fn fade_tracked(
    handle: TrackedSoundHandle,
    target_volume: f32,
    duration_ticks: u32,
) -> SjmlResult<()> {
    validate_tracked_handle(handle)?;
    validate_fade(target_volume, duration_ticks)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host resolves this handle only in the calling runtime's tracked-sound
        // registry, validates the target and duration, and schedules the fade on the client thread.
        let status = unsafe {
            imported_fade_tracked(
                handle.0 as i32,
                target_volume.to_bits() as i32,
                duration_ticks as i32,
            )
        };
        if status == 0 {
            Ok(())
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected the tracked sound fade",
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

fn validate_tracked_handle(handle: TrackedSoundHandle) -> SjmlResult<()> {
    if handle.0 == 0 || handle.0 > i32::MAX as u32 {
        return Err(invalid("tracked sound handle is invalid"));
    }
    Ok(())
}

fn validate_fade(target_volume: f32, duration_ticks: u32) -> SjmlResult<()> {
    if !target_volume.is_finite()
        || !(0.0..=MAX_SOUND_VOLUME).contains(&target_volume)
        || !(1..=MAX_SOUND_FADE_DURATION_TICKS).contains(&duration_ticks)
    {
        return Err(invalid(
            "fade target volume must be 0–4 and duration must be 1–1200 ticks",
        ));
    }
    Ok(())
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

    #[test]
    fn native_tracked_calls_fail_closed_after_validating_arguments() {
        assert_eq!(
            play_tracked(
                "minecraft:entity.player.levelup",
                SoundCategory::Players,
                1.0,
                1.0,
            )
            .unwrap_err()
            .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            play_tracked("bad ID", SoundCategory::Players, 1.0, 1.0)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            stop_tracked(TrackedSoundHandle(1)).unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            stop_tracked(TrackedSoundHandle(0)).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn validates_bounded_tracked_sound_fades() {
        assert!(validate_fade(0.0, 1).is_ok());
        assert!(validate_fade(4.0, MAX_SOUND_FADE_DURATION_TICKS).is_ok());
        for (volume, duration) in [
            (f32::NAN, 10),
            (f32::INFINITY, 10),
            (-0.01, 10),
            (4.01, 10),
            (1.0, 0),
            (1.0, MAX_SOUND_FADE_DURATION_TICKS + 1),
        ] {
            assert_eq!(
                validate_fade(volume, duration).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
        }
        assert_eq!(
            validate_tracked_handle(TrackedSoundHandle(0))
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn native_tracked_fade_fails_closed_after_validating_arguments() {
        assert_eq!(
            fade_tracked(TrackedSoundHandle(1), 1.0, 20)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            fade_tracked(TrackedSoundHandle(1), f32::NAN, 20)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }
}
