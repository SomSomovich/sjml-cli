//! Detached payload for `player.arrow_loose_attempt`.
//!
//! Register a `player.arrow_loose_attempt` callback with `events.subscribe` and
//! `events.cancel`. Returning `Ok(true)` cancels NeoForge's `ArrowLooseEvent`,
//! so no arrow or crossbow bolt leaves the bow. The callback is synchronous
//! because vanilla spawns the projectile immediately after the event returns.

use crate::{StableDecode, StableEncode};

/// Maximum UTF-8 byte length of one detached ID in an arrow-loose callback.
pub const MAX_PLAYER_ARROW_LOOSE_ATTEMPT_STRING_BYTES: usize = 256;
/// Maximum item count carried by an arrow-loose callback.
pub const MAX_PLAYER_ARROW_LOOSE_ATTEMPT_ITEM_COUNT: u32 = 1_000_000;
/// Maximum bow charge carried by an arrow-loose callback.
pub const MAX_PLAYER_ARROW_LOOSE_ATTEMPT_CHARGE: u32 = 10_000;
/// Maximum callbacks invoked for one arrow-loose attempt.
pub const MAX_PLAYER_ARROW_LOOSE_ATTEMPT_CALLBACKS: usize = 64;
/// Maximum serialized payload size for one `player.arrow_loose_attempt` callback.
pub const MAX_PLAYER_ARROW_LOOSE_ATTEMPT_PAYLOAD_BYTES: usize = 1_024;
/// Cancellation grant required to veto arrow releases; `events.subscribe` is also required.
pub const PLAYER_ARROW_LOOSE_ATTEMPT_CANCEL_CAPABILITY: &str = "events.cancel";

/// Synchronous logical-server values observed when a player releases a bow.
///
/// The host validates the player UUID, canonical dimension and item IDs, the
/// bounded stack count, the bounded charge, and the ammo flag before the
/// callback runs. Crossbows report a charge of 1 in vanilla.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerArrowLooseAttempt {
    /// Canonical UUID of the player releasing the bow.
    pub player_id: String,
    /// Resource location of the dimension containing the player.
    pub dimension_id: String,
    /// Registry ID of the released bow or crossbow item.
    pub item_id: String,
    /// Stack count of the released bow in `0..=1_000_000`.
    pub count: u32,
    /// Charge measured by vanilla in `0..=10_000`.
    pub charge: u32,
    /// Whether the player had a projectile available to fire.
    pub has_ammo: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_bytes, to_bytes, ErrorCode};

    fn sample() -> PlayerArrowLooseAttempt {
        PlayerArrowLooseAttempt {
            player_id: "00000000-0000-0000-0000-000000000001".into(),
            dimension_id: "minecraft:overworld".into(),
            item_id: "minecraft:bow".into(),
            count: 1,
            charge: 20,
            has_ammo: true,
        }
    }

    fn expected_wire(event: &PlayerArrowLooseAttempt) -> Vec<u8> {
        let mut output = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut output, &event.player_id);
        append_test_string(&mut output, &event.dimension_id);
        append_test_string(&mut output, &event.item_id);
        output.extend_from_slice(&event.count.to_le_bytes());
        output.extend_from_slice(&event.charge.to_le_bytes());
        output.push(u8::from(event.has_ammo));
        output
    }

    #[test]
    fn arrow_loose_attempt_matches_the_java_wire_layout_and_round_trips() {
        let event = sample();
        let expected = expected_wire(&event);
        assert_eq!(to_bytes(&event).unwrap(), expected);
        assert_eq!(
            from_bytes::<PlayerArrowLooseAttempt>(&expected).unwrap(),
            event
        );

        let mut empty = sample();
        empty.count = 0;
        empty.charge = 0;
        empty.has_ammo = false;
        let expected = expected_wire(&empty);
        assert_eq!(to_bytes(&empty).unwrap(), expected);
        assert_eq!(
            from_bytes::<PlayerArrowLooseAttempt>(&expected).unwrap(),
            empty
        );
    }

    #[test]
    fn arrow_loose_attempt_rejects_a_foreign_schema_and_boolean_bytes() {
        let mut wrong_version = to_bytes(&sample()).unwrap();
        wrong_version[..4].copy_from_slice(&2_i32.to_le_bytes());
        assert_eq!(
            from_bytes::<PlayerArrowLooseAttempt>(&wrong_version)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );

        let event = sample();
        let mut wrong_boolean = to_bytes(&event).unwrap();
        let flag_offset = 4
            + (4 + event.player_id.len())
            + (4 + event.dimension_id.len())
            + (4 + event.item_id.len())
            + 4
            + 4;
        wrong_boolean[flag_offset] = 2;
        assert_eq!(
            from_bytes::<PlayerArrowLooseAttempt>(&wrong_boolean)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidBoolean
        );
    }

    fn append_test_string(output: &mut Vec<u8>, value: &str) {
        output.extend_from_slice(&(value.len() as i32).to_le_bytes());
        output.extend_from_slice(value.as_bytes());
    }
}
