//! Bounded situational music selection on the physical client.
//!
//! Modules opt in with `client.audio.music` and `subscribe_music_selection()`. The host invokes
//! the `#[client_music_selection]` callback synchronously before vanilla situational music starts.
//! Its detached input contains only resource IDs, delay integers, and a replacement flag. A module
//! can keep the current vanilla selection, stop situational music, or select a sound event that
//! the same module successfully registered.

use crate::{ErrorCode, SjmlError, SjmlResult, StableDecode, StableEncode};

/// Capability required to subscribe to physical-client situational music selection.
pub const CLIENT_MUSIC_CAPABILITY: &str = "client.audio.music";
/// Maximum length of a resource ID used as a music track, in UTF-8 bytes.
pub const MAX_MUSIC_SOUND_ID_BYTES: usize = 256;
/// Largest replacement delay accepted for a situational music selector, in client ticks.
pub const MAX_MUSIC_DELAY_TICKS: u32 = 1_200;

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_client_music_v1")]
extern "C" {
    #[link_name = "subscribe"]
    fn imported_subscribe() -> i32;

    #[link_name = "unsubscribe"]
    fn imported_unsubscribe() -> i32;
}

/// Detached snapshot supplied immediately before the host selects situational music.
#[derive(Clone, Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct ClientMusicSelectionEvent {
    /// Resource ID of the vanilla or mod-provided situational track selected by Minecraft.
    pub original_sound_id: String,
    /// Resource ID of the track currently playing, if any.
    pub current_music_id: Option<String>,
    /// Minimum delay attached to the original Minecraft selector, in ticks.
    pub original_min_delay_ticks: i32,
    /// Maximum delay attached to the original Minecraft selector, in ticks.
    pub original_max_delay_ticks: i32,
    /// Whether Minecraft's original selector may replace the current track immediately.
    pub original_replace_current: bool,
}

/// A bounded choice returned by a `#[client_music_selection]` callback.
#[derive(Clone, Debug, PartialEq, StableEncode)]
#[sjml(version = 1)]
pub struct MusicSelectionDecision {
    action: u8,
    sound_id: Option<String>,
    min_delay_ticks: u32,
    max_delay_ticks: u32,
    replace_current: bool,
}

impl MusicSelectionDecision {
    /// Leave the currently selected vanilla or earlier SJML result unchanged.
    pub const fn keep() -> Self {
        Self {
            action: 0,
            sound_id: None,
            min_delay_ticks: 0,
            max_delay_ticks: 0,
            replace_current: false,
        }
    }

    /// Stop any currently playing situational music.
    pub const fn stop() -> Self {
        Self {
            action: 1,
            sound_id: None,
            min_delay_ticks: 0,
            max_delay_ticks: 0,
            replace_current: false,
        }
    }

    /// Select a sound registered by this module, with a bounded delay range.
    pub fn replace(
        sound_id: impl Into<String>,
        min_delay_ticks: u32,
        max_delay_ticks: u32,
        replace_current: bool,
    ) -> SjmlResult<Self> {
        let sound_id = sound_id.into();
        validate_sound_id(&sound_id)?;
        if min_delay_ticks > max_delay_ticks || max_delay_ticks > MAX_MUSIC_DELAY_TICKS {
            return Err(invalid(
                "music delay must satisfy 0 <= min <= max <= 1200 ticks",
            ));
        }
        Ok(Self {
            action: 2,
            sound_id: Some(sound_id),
            min_delay_ticks,
            max_delay_ticks,
            replace_current,
        })
    }
}

/// Registers the module's `#[client_music_selection]` callback for this runtime.
///
/// The module must declare and receive `client.audio.music`. It may subscribe once; a second
/// subscription returns `ServiceBusy`. Call `unsubscribe_music_selection` to stop receiving
/// callbacks. Runtime teardown also removes the subscription.
pub fn subscribe_music_selection() -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let status = unsafe { imported_subscribe() };
        if status == 0 {
            Ok(())
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected music-selection subscription",
            ))
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(SjmlError::new(
            ErrorCode::UnknownErrorCode,
            "client music selection is available only to wasm32 guests",
        ))
    }
}

/// Removes the module's music-selection callback before runtime teardown.
pub fn unsubscribe_music_selection() -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let status = unsafe { imported_unsubscribe() };
        if status == 0 {
            Ok(())
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected music-selection unsubscription",
            ))
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(SjmlError::new(
            ErrorCode::UnknownErrorCode,
            "client music selection is available only to wasm32 guests",
        ))
    }
}

fn validate_sound_id(sound_id: &str) -> SjmlResult<()> {
    if sound_id.len() < 3 || sound_id.len() > MAX_MUSIC_SOUND_ID_BYTES {
        return Err(invalid("music sound ID must be 3–256 UTF-8 bytes"));
    }
    let Some((namespace, path)) = sound_id.split_once(':') else {
        return Err(invalid("music sound ID must use namespace:path form"));
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
            "music sound ID must be canonical namespace:path syntax",
        ));
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
    fn creates_keep_stop_and_bounded_replacement_decisions() {
        assert_eq!(MusicSelectionDecision::keep().action, 0);
        assert_eq!(MusicSelectionDecision::stop().action, 1);
        let replace =
            MusicSelectionDecision::replace("example:music/track", 20, 1_200, true).unwrap();
        assert_eq!(replace.action, 2);
        assert_eq!(replace.sound_id.as_deref(), Some("example:music/track"));
        assert!(replace.replace_current);
    }

    #[test]
    fn stable_callback_records_keep_the_declared_little_endian_field_order() {
        let event = ClientMusicSelectionEvent {
            original_sound_id: "minecraft:music.game".into(),
            current_music_id: Some("example:music/night".into()),
            original_min_delay_ticks: 100,
            original_max_delay_ticks: 300,
            original_replace_current: true,
        };
        let mut expected = Vec::new();
        expected.extend_from_slice(&1u32.to_le_bytes());
        expected.extend_from_slice(&20u32.to_le_bytes());
        expected.extend_from_slice(b"minecraft:music.game");
        expected.push(1);
        expected.extend_from_slice(&19u32.to_le_bytes());
        expected.extend_from_slice(b"example:music/night");
        expected.extend_from_slice(&100i32.to_le_bytes());
        expected.extend_from_slice(&300i32.to_le_bytes());
        expected.push(1);
        assert_eq!(crate::to_bytes(&event).unwrap(), expected);

        let decision =
            MusicSelectionDecision::replace("example:music/night", 20, 1_200, true).unwrap();
        let mut expected_decision = Vec::new();
        expected_decision.extend_from_slice(&1u32.to_le_bytes());
        expected_decision.push(2);
        expected_decision.push(1);
        expected_decision.extend_from_slice(&19u32.to_le_bytes());
        expected_decision.extend_from_slice(b"example:music/night");
        expected_decision.extend_from_slice(&20u32.to_le_bytes());
        expected_decision.extend_from_slice(&1_200u32.to_le_bytes());
        expected_decision.push(1);
        assert_eq!(crate::to_bytes(&decision).unwrap(), expected_decision);
    }

    #[test]
    fn rejects_unregistered_shape_and_out_of_bound_music_requests_early() {
        for id in [
            "bad ID",
            "minecraft:/music",
            "example:a/../b",
            "example:music//track",
        ] {
            assert_eq!(
                MusicSelectionDecision::replace(id, 0, 20, false)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument
            );
        }
        assert_eq!(
            MusicSelectionDecision::replace("example:track", 21, 20, false)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            MusicSelectionDecision::replace("example:track", 0, 1_201, false)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            MusicSelectionDecision::replace("example:track", 0, 20, false)
                .unwrap()
                .max_delay_ticks,
            20
        );
    }

    #[test]
    fn native_subscription_fails_closed() {
        assert_eq!(
            subscribe_music_selection().unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            unsubscribe_music_selection().unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
    }
}
