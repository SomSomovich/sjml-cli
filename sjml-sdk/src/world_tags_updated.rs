//! Detached observation of a logical-server registry tag reload.
//!
//! Subscribe with `events::subscribe("world.tags_updated", EventPriority::Normal)` and a typed
//! `#[event(name = "world.tags_updated")]` callback returning `SjmlResult<()>`. The notification is
//! observational: it cannot cancel or change the tag reload. Callbacks run on the owning server
//! thread, and the host invokes at most [`MAX_CALLBACKS_PER_EVENT`] callbacks per update.
//!
//! The host only dispatches this event for logical-server contexts, even though NeoForge may also
//! post the underlying hook on clients.

use sjml::{StableDecode, StableEncode};

/// Event name used with [`crate::events::subscribe`].
pub const EVENT_NAME: &str = "world.tags_updated";
/// Generated ABI export name for the typed tag-update callback.
pub const EXPORT_NAME: &str = "sjml_v1_event_world.tags_updated";
/// Maximum UTF-8 byte length of the cause field.
pub const MAX_STRING_BYTES: usize = 64;
/// Maximum encoded bytes for one detached tag-update event.
pub const MAX_PAYLOAD_BYTES: usize = 256;
/// Maximum guest callbacks invoked for one tag-update event.
pub const MAX_CALLBACKS_PER_EVENT: usize = 64;

/// Canonical cause for tags reloaded by the server from datapacks.
pub const CAUSE_SERVER_DATA_LOAD: &str = "server_data_load";
/// Canonical cause for tags received by a client from its server.
pub const CAUSE_CLIENT_PACKET_RECEIVED: &str = "client_packet_received";

/// Detached tag-update notification. It contains no level, registry, or host handle.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct WorldTagsUpdated {
    /// Cause of the update: `server_data_load` or `client_packet_received`.
    pub cause: String,
    /// Whether the update was triggered by a client packet.
    pub from_client_packet: bool,
    /// Whether the packet came from an integrated server connection.
    pub integrated_server_connection: bool,
}

#[cfg(test)]
mod tests {
    use super::{
        WorldTagsUpdated, CAUSE_CLIENT_PACKET_RECEIVED, CAUSE_SERVER_DATA_LOAD, EVENT_NAME,
        EXPORT_NAME, MAX_CALLBACKS_PER_EVENT, MAX_PAYLOAD_BYTES, MAX_STRING_BYTES,
    };
    use sjml::{from_bytes, to_bytes, ErrorCode};

    fn event() -> WorldTagsUpdated {
        WorldTagsUpdated {
            cause: CAUSE_SERVER_DATA_LOAD.into(),
            from_client_packet: false,
            integrated_server_connection: false,
        }
    }

    #[test]
    fn wire_bytes_match_schema_v1_field_order() {
        let original = event();
        let bytes = to_bytes(&original).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_u32.to_le_bytes());
        expected.extend_from_slice(&(original.cause.len() as u32).to_le_bytes());
        expected.extend_from_slice(original.cause.as_bytes());
        expected.push(0);
        expected.push(0);
        assert_eq!(bytes, expected);
        assert!(bytes.len() <= MAX_PAYLOAD_BYTES);
    }

    #[test]
    fn stable_round_trip_preserves_cause_and_flags() {
        let original = WorldTagsUpdated {
            cause: CAUSE_CLIENT_PACKET_RECEIVED.into(),
            from_client_packet: true,
            integrated_server_connection: true,
        };
        let bytes = to_bytes(&original).unwrap();
        assert_eq!(
            u32::from_le_bytes(bytes[0..4].try_into().unwrap()),
            1,
            "the schema version must be the first little-endian word"
        );
        assert_eq!(bytes[bytes.len() - 2], 1);
        assert_eq!(bytes[bytes.len() - 1], 1);
        assert_eq!(from_bytes::<WorldTagsUpdated>(&bytes).unwrap(), original);
    }

    #[test]
    fn codec_rejects_unsupported_schema_invalid_boolean_and_trailing_bytes() {
        let bytes = to_bytes(&event()).unwrap();

        let mut bad_schema = bytes.clone();
        bad_schema[0..4].copy_from_slice(&2_u32.to_le_bytes());
        assert_eq!(
            from_bytes::<WorldTagsUpdated>(&bad_schema)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );

        let mut bad_boolean = bytes.clone();
        let last = bad_boolean.len() - 1;
        bad_boolean[last] = 2;
        assert_eq!(
            from_bytes::<WorldTagsUpdated>(&bad_boolean)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidBoolean
        );

        let mut trailing = bytes;
        trailing.push(0);
        assert!(from_bytes::<WorldTagsUpdated>(&trailing).is_err());
    }

    #[test]
    fn published_limits_match_the_host_contract() {
        assert_eq!("world.tags_updated", EVENT_NAME);
        assert_eq!("sjml_v1_event_world.tags_updated", EXPORT_NAME);
        assert_eq!("server_data_load", CAUSE_SERVER_DATA_LOAD);
        assert_eq!("client_packet_received", CAUSE_CLIENT_PACKET_RECEIVED);
        assert_eq!(64, MAX_STRING_BYTES);
        assert_eq!(256, MAX_PAYLOAD_BYTES);
        assert_eq!(64, MAX_CALLBACKS_PER_EVENT);
    }
}
