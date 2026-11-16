//! Detached observation of a logical-server player smelting an item.
//!
//! Subscribe with `events::subscribe("player.item_smelted", EventPriority::Normal)` and a typed
//! `#[event(name = "player.item_smelted")]` callback returning `SjmlResult<()>`. The notification
//! is observational: it cannot cancel or change the smelting result. Callbacks run on the owning
//! server thread, and the host invokes at most [`MAX_CALLBACKS`] callbacks per smelt.

use sjml::{StableDecode, StableEncode};

/// Event name used with [`crate::events::subscribe`].
pub const EVENT_NAME: &str = "player.item_smelted";
/// Generated ABI export name for the typed smelting callback.
pub const EXPORT_NAME: &str = "sjml_v1_event_player.item_smelted";
/// Maximum UTF-8 byte length of the player UUID, dimension ID, and item ID.
pub const MAX_STRING_BYTES: usize = 256;
/// Maximum smelted item count accepted by the host.
pub const MAX_ITEM_COUNT: i32 = 1_000_000;
/// Maximum encoded bytes for one detached smelting event.
pub const MAX_PAYLOAD_BYTES: usize = 768;
/// Maximum guest callbacks invoked for one smelting event.
pub const MAX_CALLBACKS: usize = 64;

/// Detached smelting notification. It contains no `Player`, level, or item-stack handle.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerItemSmelted {
    /// Canonical lowercase UUID of the player that smelted the item.
    pub player_id: String,
    /// Resource location of the dimension containing the player.
    pub dimension_id: String,
    /// Canonical registry ID of the smelted item.
    pub item_id: String,
    /// Smelted item count, bounded by the host to `1..=1_000_000`.
    pub count: i32,
}

#[cfg(test)]
mod tests {
    use super::{
        PlayerItemSmelted, EVENT_NAME, EXPORT_NAME, MAX_CALLBACKS, MAX_ITEM_COUNT,
        MAX_PAYLOAD_BYTES, MAX_STRING_BYTES,
    };
    use sjml::{from_bytes, to_bytes, ErrorCode};

    fn event() -> PlayerItemSmelted {
        PlayerItemSmelted {
            player_id: "123e4567-e89b-12d3-a456-426614174000".into(),
            dimension_id: "minecraft:overworld".into(),
            item_id: "minecraft:iron_ingot".into(),
            count: 2,
        }
    }

    #[test]
    fn wire_bytes_match_schema_v1_field_order() {
        let original = event();
        let bytes = to_bytes(&original).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_i32.to_le_bytes());
        for value in [
            &original.player_id,
            &original.dimension_id,
            &original.item_id,
        ] {
            expected.extend_from_slice(&(value.len() as u32).to_le_bytes());
            expected.extend_from_slice(value.as_bytes());
        }
        expected.extend_from_slice(&original.count.to_le_bytes());
        assert_eq!(bytes, expected);
        assert!(bytes.len() <= MAX_PAYLOAD_BYTES);
    }

    #[test]
    fn stable_round_trip_preserves_the_smelted_item() {
        let original = PlayerItemSmelted {
            item_id: "minecraft:gold_ingot".into(),
            count: MAX_ITEM_COUNT,
            ..event()
        };
        let bytes = to_bytes(&original).unwrap();
        assert_eq!(
            u32::from_le_bytes(bytes[0..4].try_into().unwrap()),
            1,
            "the schema version must be the first little-endian word"
        );
        assert_eq!(from_bytes::<PlayerItemSmelted>(&bytes).unwrap(), original);
    }

    #[test]
    fn codec_rejects_unsupported_schema_and_trailing_bytes() {
        let bytes = to_bytes(&event()).unwrap();

        let mut bad_schema = bytes.clone();
        bad_schema[0..4].copy_from_slice(&2_i32.to_le_bytes());
        assert_eq!(
            from_bytes::<PlayerItemSmelted>(&bad_schema)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );

        let mut trailing = bytes;
        trailing.push(0);
        assert!(from_bytes::<PlayerItemSmelted>(&trailing).is_err());
    }

    #[test]
    fn published_limits_match_the_host_contract() {
        assert_eq!("player.item_smelted", EVENT_NAME);
        assert_eq!("sjml_v1_event_player.item_smelted", EXPORT_NAME);
        assert_eq!(256, MAX_STRING_BYTES);
        assert_eq!(1_000_000, MAX_ITEM_COUNT);
        assert_eq!(768, MAX_PAYLOAD_BYTES);
        assert_eq!(64, MAX_CALLBACKS);
    }
}
