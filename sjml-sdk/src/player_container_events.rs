//! Detached observation of server players opening and closing container menus.

use sjml::{StableDecode, StableEncode};

/// Separate grant required to observe server-player container open and close events.
pub const OBSERVE_CAPABILITY: &str = "player.container.observe";
/// Maximum UTF-8 byte length of the player, dimension, and menu strings.
pub const MAX_STRING_BYTES: usize = 256;
/// Maximum container ID reported by one container event.
pub const MAX_CONTAINER_ID: i32 = 255;
/// Maximum synchronized slot count reported by one container event.
pub const MAX_SLOT_COUNT: i32 = 512;
/// Maximum callbacks the host invokes for one container open or close event.
pub const MAX_CALLBACKS: usize = 64;
/// Maximum encoded payload size for one container open or close callback.
pub const MAX_PAYLOAD_BYTES: usize = 2_048;

/// Observation-only snapshot emitted when a server player opens a container menu.
///
/// NeoForge fires `PlayerContainerEvent.Open` on both logical sides; SJML
/// dispatches only on the owning logical-server thread. The notification is not
/// cancellable, so the handler returns `()` rather than a veto. All values are
/// detached; no `Player`, menu, or level reference crosses the guest ABI.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerContainerOpened {
    /// Canonical UUID of the player who opened the menu.
    pub player_id: String,
    /// Resource location of the player's dimension.
    pub dimension_id: String,
    /// Canonical registry ID of the opened menu type.
    pub menu_id: String,
    /// Synchronized container ID in the `0..=255` range.
    pub container_id: i32,
    /// Number of synchronized menu slots in the `0..=512` range.
    pub slot_count: i32,
}

/// Observation-only snapshot emitted when a server player closes a container menu.
///
/// NeoForge fires `PlayerContainerEvent.Close` on both logical sides; SJML
/// dispatches only on the owning logical-server thread. The notification is not
/// cancellable, so the handler returns `()` rather than a veto. All values are
/// detached; no `Player`, menu, or level reference crosses the guest ABI.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerContainerClosed {
    /// Canonical UUID of the player who closed the menu.
    pub player_id: String,
    /// Resource location of the player's dimension.
    pub dimension_id: String,
    /// Canonical registry ID of the closed menu type.
    pub menu_id: String,
    /// Synchronized container ID in the `0..=255` range.
    pub container_id: i32,
    /// Number of synchronized menu slots in the `0..=512` range.
    pub slot_count: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sjml::{from_bytes, to_bytes};

    fn opened() -> PlayerContainerOpened {
        PlayerContainerOpened {
            player_id: "00000000-0000-0000-0000-000000000001".into(),
            dimension_id: "minecraft:overworld".into(),
            menu_id: "minecraft:generic_9x1".into(),
            container_id: 3,
            slot_count: 45,
        }
    }

    fn closed() -> PlayerContainerClosed {
        PlayerContainerClosed {
            player_id: "00000000-0000-0000-0000-000000000002".into(),
            dimension_id: "minecraft:the_nether".into(),
            menu_id: "minecraft:shulker_box".into(),
            container_id: 7,
            slot_count: 27,
        }
    }

    #[test]
    fn container_opened_matches_the_java_wire_layout() {
        let event = opened();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &event.player_id);
        append_test_string(&mut expected, &event.dimension_id);
        append_test_string(&mut expected, &event.menu_id);
        expected.extend_from_slice(&event.container_id.to_le_bytes());
        expected.extend_from_slice(&event.slot_count.to_le_bytes());

        assert_eq!(to_bytes(&event).unwrap(), expected);
        assert_eq!(
            from_bytes::<PlayerContainerOpened>(&expected).unwrap(),
            event
        );
    }

    #[test]
    fn container_closed_matches_the_java_wire_layout() {
        let event = closed();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &event.player_id);
        append_test_string(&mut expected, &event.dimension_id);
        append_test_string(&mut expected, &event.menu_id);
        expected.extend_from_slice(&event.container_id.to_le_bytes());
        expected.extend_from_slice(&event.slot_count.to_le_bytes());

        assert_eq!(to_bytes(&event).unwrap(), expected);
        assert_eq!(
            from_bytes::<PlayerContainerClosed>(&expected).unwrap(),
            event
        );
    }

    fn append_test_string(output: &mut Vec<u8>, value: &str) {
        output.extend_from_slice(&(value.len() as i32).to_le_bytes());
        output.extend_from_slice(value.as_bytes());
    }
}
