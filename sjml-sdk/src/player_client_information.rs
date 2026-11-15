//! Detached observation of a logical-server player's updated client settings.
//!
//! Subscribe with `events::subscribe("player.client_information", EventPriority::Normal)` and a
//! typed `#[event(name = "player.client_information")]` callback returning `SjmlResult<()>`. The
//! notification is observational: it cannot cancel or change the client-information update.
//! Callbacks run on the owning server thread, and the host invokes at most
//! [`MAX_CALLBACKS_PER_EVENT`] callbacks per update.

use sjml::{StableDecode, StableEncode};

/// Event name used with [`crate::events::subscribe`].
pub const EVENT_NAME: &str = "player.client_information";
/// Generated ABI export name for the typed client-information callback.
pub const EXPORT_NAME: &str = "sjml_v1_event_player.client_information";
/// Maximum UTF-8 byte length of the player UUID, dimension ID, and enum names.
pub const MAX_STRING_BYTES: usize = 256;
/// Maximum UTF-8 byte length of the language code.
pub const MAX_LANGUAGE_BYTES: usize = 64;
/// Maximum accepted view distance.
pub const MAX_VIEW_DISTANCE: i32 = 32;
/// Maximum accepted model customisation value.
pub const MAX_MODEL_CUSTOMISATION: i32 = 255;
/// Maximum encoded bytes for one detached client-information event.
pub const MAX_PAYLOAD_BYTES: usize = 1_024;
/// Maximum guest callbacks invoked for one client-information event.
pub const MAX_CALLBACKS_PER_EVENT: usize = 64;

/// Detached client-information notification. It contains no `Player`, level, or host handle.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerClientInformation {
    /// Canonical lowercase UUID of the player that updated its settings.
    pub player_id: String,
    /// Resource location of the dimension containing the player.
    pub dimension_id: String,
    /// Language code selected by the client, without control characters.
    pub language: String,
    /// Requested server view distance, between 0 and 32.
    pub view_distance: i32,
    /// Lowercase chat-visibility enum name, such as `full` or `hidden`.
    pub chat_visibility: String,
    /// Whether chat colors are enabled.
    pub chat_colors: bool,
    /// Requested model customisation value, between 0 and 255.
    pub model_customisation: i32,
    /// Lowercase main-hand enum name, such as `left` or `right`.
    pub main_hand: String,
    /// Whether the client requested filtered text.
    pub text_filtering_enabled: bool,
    /// Whether the player may appear in the server listing.
    pub allows_listing: bool,
}

#[cfg(test)]
mod tests {
    use super::{
        PlayerClientInformation, EVENT_NAME, EXPORT_NAME, MAX_CALLBACKS_PER_EVENT,
        MAX_LANGUAGE_BYTES, MAX_MODEL_CUSTOMISATION, MAX_PAYLOAD_BYTES, MAX_STRING_BYTES,
        MAX_VIEW_DISTANCE,
    };
    use sjml::{from_bytes, to_bytes, ErrorCode};

    fn event() -> PlayerClientInformation {
        PlayerClientInformation {
            player_id: "123e4567-e89b-12d3-a456-426614174000".into(),
            dimension_id: "minecraft:overworld".into(),
            language: "en_us".into(),
            view_distance: 12,
            chat_visibility: "full".into(),
            chat_colors: true,
            model_customisation: 0,
            main_hand: "right".into(),
            text_filtering_enabled: false,
            allows_listing: true,
        }
    }

    #[test]
    fn wire_bytes_match_schema_v1_field_order() {
        let original = event();
        let bytes = to_bytes(&original).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_u32.to_le_bytes());
        for value in [
            &original.player_id,
            &original.dimension_id,
            &original.language,
        ] {
            expected.extend_from_slice(&(value.len() as u32).to_le_bytes());
            expected.extend_from_slice(value.as_bytes());
        }
        expected.extend_from_slice(&original.view_distance.to_le_bytes());
        expected.extend_from_slice(&(original.chat_visibility.len() as u32).to_le_bytes());
        expected.extend_from_slice(original.chat_visibility.as_bytes());
        expected.push(u8::from(original.chat_colors));
        expected.extend_from_slice(&original.model_customisation.to_le_bytes());
        expected.extend_from_slice(&(original.main_hand.len() as u32).to_le_bytes());
        expected.extend_from_slice(original.main_hand.as_bytes());
        expected.push(u8::from(original.text_filtering_enabled));
        expected.push(u8::from(original.allows_listing));
        assert_eq!(bytes, expected);
        assert!(bytes.len() <= MAX_PAYLOAD_BYTES);
    }

    #[test]
    fn stable_round_trip_preserves_detached_client_information() {
        let original = PlayerClientInformation {
            language: "de_de".into(),
            view_distance: 32,
            chat_visibility: "hidden".into(),
            chat_colors: false,
            model_customisation: 255,
            main_hand: "left".into(),
            text_filtering_enabled: true,
            allows_listing: false,
            ..event()
        };
        let bytes = to_bytes(&original).unwrap();
        assert_eq!(
            u32::from_le_bytes(bytes[0..4].try_into().unwrap()),
            1,
            "the schema version must be the first little-endian word"
        );
        assert_eq!(
            from_bytes::<PlayerClientInformation>(&bytes).unwrap(),
            original
        );
    }

    #[test]
    fn codec_rejects_unsupported_schema_invalid_boolean_and_trailing_bytes() {
        let bytes = to_bytes(&event()).unwrap();

        let mut bad_schema = bytes.clone();
        bad_schema[0..4].copy_from_slice(&2_u32.to_le_bytes());
        assert_eq!(
            from_bytes::<PlayerClientInformation>(&bad_schema)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );

        let mut bad_boolean = bytes.clone();
        let last = bad_boolean.len() - 1;
        bad_boolean[last] = 2;
        assert_eq!(
            from_bytes::<PlayerClientInformation>(&bad_boolean)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidBoolean
        );

        let mut trailing = bytes;
        trailing.push(0);
        assert!(from_bytes::<PlayerClientInformation>(&trailing).is_err());
    }

    #[test]
    fn published_limits_match_the_host_contract() {
        assert_eq!("player.client_information", EVENT_NAME);
        assert_eq!("sjml_v1_event_player.client_information", EXPORT_NAME);
        assert_eq!(256, MAX_STRING_BYTES);
        assert_eq!(64, MAX_LANGUAGE_BYTES);
        assert_eq!(32, MAX_VIEW_DISTANCE);
        assert_eq!(255, MAX_MODEL_CUSTOMISATION);
        assert_eq!(1_024, MAX_PAYLOAD_BYTES);
        assert_eq!(64, MAX_CALLBACKS_PER_EVENT);
    }
}
