//! Stable payload records used by the built-in NeoForge event adapters.

use crate::abi::EventSubscriptionHandle;
#[cfg(target_arch = "wasm32")]
use crate::abi::{allocate_host_input, deallocate_host_input};
use crate::inventory::BlockPos;
#[cfg(target_arch = "wasm32")]
use crate::{from_bytes, to_bytes};
use crate::{ErrorCode, SjmlError, SjmlResult, StableDecode, StableEncode};

/// Default `server.tick` subscription cadence, in server ticks.
pub const DEFAULT_SERVER_TICK_INTERVAL: u32 = 20;
/// Maximum supported `server.tick` subscription cadence, in server ticks.
pub const MAX_SERVER_TICK_INTERVAL: u32 = 1_200;
/// Maximum item count represented by one `inventory.changed` notification.
pub const MAX_INVENTORY_CHANGE_COUNT: i32 = 4_096;
/// Maximum player inventory slot index represented by `inventory.changed`.
pub const MAX_INVENTORY_CHANGE_SLOT: i32 = 255;

/// Synchronous, observation-only snapshot of a logical-server player interaction attempt.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerInteracted {
    /// Canonical UUID string of the player who initiated the interaction.
    pub player_id: String,
    /// Resource location of the level where the interaction occurred.
    pub dimension_id: String,
    /// Interaction kind, such as `right_click_block` or `entity_interact`.
    pub action: String,
    /// `main_hand` or `off_hand`.
    pub hand: String,
    /// Registry ID of the block, entity type, or item targeted, when available.
    pub target_id: Option<String>,
    /// Block face for block interactions, when available.
    pub face: Option<String>,
    /// Target block position for block interactions, when available.
    pub block_position: Option<BlockPos>,
}

/// Synchronous, non-cancellable snapshot emitted after damage has been applied to a living entity.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityDamaged {
    /// Canonical UUID string of the damaged living entity.
    pub entity_id: String,
    /// Registry ID of the damaged entity type.
    pub entity_type_id: String,
    /// Resource location of the entity's level.
    pub dimension_id: String,
    /// Registry ID of the damage type.
    pub damage_type_id: String,
    /// UUID of the responsible source entity, when the damage had one.
    pub source_entity_id: Option<String>,
    /// Entity X coordinate after damage was applied.
    pub x: f64,
    /// Entity Y coordinate after damage was applied.
    pub y: f64,
    /// Entity Z coordinate after damage was applied.
    pub z: f64,
    /// Damage before armor and other mitigation.
    pub original_damage: f32,
    /// Damage that was actually applied after mitigation.
    pub applied_damage: f32,
}

/// Synchronous lifecycle notification for one logical-server level.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct WorldLifecycle {
    /// Resource location identifying the world or dimension.
    pub dimension_id: String,
    /// World game time when the lifecycle callback was delivered.
    pub game_time: i64,
}

/// Synchronous, non-cancellable notification emitted after a scheduled server tick.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct ServerTick {
    /// Logical server tick count when this callback was delivered.
    pub tick_count: i64,
}

/// Synchronous, non-cancellable snapshot of one successful player inventory change.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct InventoryChanged {
    /// Canonical UUID string of the player whose inventory changed.
    pub player_id: String,
    /// Registry ID of the item affected by the change.
    pub item_id: String,
    /// Operation kind: `picked_up`, `crafted`, or `dropped`.
    pub change_type: String,
    /// Player inventory slot index when the hook exposes it; otherwise `None`.
    pub slot: Option<i32>,
    /// Number of items successfully added to or removed from the inventory.
    pub count: i32,
}

/// Synchronous, non-cancellable notification emitted after a server resource reload commits.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct ResourcesReloaded {
    /// Server tick observed after the newly loaded server resources became active.
    pub tick_count: i64,
}

/// Cancellable logical-server `block.break` snapshot, delivered synchronously on the server thread.
/// A handler returns `SjmlResult<bool>`; `true` vetoes the break when the module has both
/// `events.subscribe` and `events.cancel` granted.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct BlockBreak {
    /// Canonical UUID string of the player who attempted to break the block.
    pub player_id: String,
    /// Resource location of the owning dimension, such as `minecraft:overworld`.
    pub dimension_id: String,
    /// Resource location of the block state, such as `minecraft:stone`.
    pub block_id: String,
    /// Signed block X coordinate.
    pub x: i32,
    /// Signed block Y coordinate.
    pub y: i32,
    /// Signed block Z coordinate.
    pub z: i32,
}

/// Synchronous, non-cancellable `block.place` snapshot from the logical server.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct BlockPlace {
    /// UUID of the entity that placed the block, if the placement had an entity actor.
    pub placer_id: Option<String>,
    /// Resource location of the owning dimension.
    pub dimension_id: String,
    /// Registry ID of the placed block.
    pub block_id: String,
    /// Signed block X coordinate.
    pub x: i32,
    /// Signed block Y coordinate.
    pub y: i32,
    /// Signed block Z coordinate.
    pub z: i32,
}

/// Player identity delivered by the synchronous, non-cancellable `player.joined` event.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerJoined {
    /// Canonical UUID string of the player who joined.
    pub player_id: String,
    /// Minecraft profile name captured on the logical server thread.
    pub display_name: String,
}

/// Player identity delivered by the synchronous, non-cancellable `player.left` event.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerLeft {
    /// Canonical UUID string of the player who left.
    pub player_id: String,
    /// Minecraft profile name captured on the logical server thread.
    pub display_name: String,
}

/// Synchronous, non-cancellable `entity.death` snapshot from the logical server.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityDeath {
    /// Canonical lowercase UUID of the living entity.
    pub entity_id: String,
    /// Canonical registry ID of the entity type.
    pub entity_type_id: String,
    /// Canonical dimension ID where the death event fired.
    pub dimension_id: String,
    /// Entity X coordinate at the event.
    pub x: f64,
    /// Entity Y coordinate at the event.
    pub y: f64,
    /// Entity Z coordinate at the event.
    pub z: f64,
}

/// Synchronous, non-cancellable entity join notification from the logical server.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntitySpawned {
    /// Canonical lowercase UUID of the entity.
    pub entity_id: String,
    /// Canonical registry ID of the entity type.
    pub entity_type_id: String,
    /// Canonical dimension ID where the entity joined the level.
    pub dimension_id: String,
    /// Entity X coordinate at the event.
    pub x: f64,
    /// Entity Y coordinate at the event.
    pub y: f64,
    /// Entity Z coordinate at the event.
    pub z: f64,
    /// Whether the entity was restored from saved world data rather than freshly spawned.
    pub loaded_from_disk: bool,
}

/// Synchronous `entity.effect.tick` snapshot emitted at a registered interval for one active effect.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EffectTickEvent {
    /// Registry ID of the active effect.
    pub effect_id: String,
    /// Canonical lowercase UUID of the affected living entity.
    pub entity_id: String,
    /// Registry ID of the affected entity type.
    pub entity_type_id: String,
    /// Registry ID of the dimension containing the entity.
    pub dimension_id: String,
    /// Entity X coordinate at the end of this tick.
    pub x: f64,
    /// Entity Y coordinate at the end of this tick.
    pub y: f64,
    /// Entity Z coordinate at the end of this tick.
    pub z: f64,
    /// Zero-based effect amplifier.
    pub amplifier: i32,
    /// Remaining effect duration in ticks; `-1` represents an infinite effect.
    pub duration_ticks: i32,
    /// Configured number of entity ticks between callbacks.
    pub tick_interval: u32,
}

/// Callback order for handlers subscribed to the same synchronous event.
/// Lower order values run first; equal priorities retain subscription order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum EventPriority {
    Highest = 0,
    High = 1,
    Normal = 2,
    Low = 3,
    Lowest = 4,
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_events_v1")]
extern "C" {
    #[link_name = "subscribe"]
    fn imported_subscribe(
        event_pointer: i32,
        event_length: i32,
        priority: i32,
        handle_output_pointer: i32,
    ) -> i32;
    #[link_name = "subscribe_interval"]
    fn imported_subscribe_interval(
        event_pointer: i32,
        event_length: i32,
        priority: i32,
        interval_ticks: i32,
        handle_output_pointer: i32,
    ) -> i32;
    #[link_name = "unsubscribe"]
    fn imported_unsubscribe(handle_pointer: i32, handle_length: i32) -> i32;
}

/// Registers this module's `#[event(name = "...")]` or `#[event_async(name = "...")]`
/// callback and returns its scoped handle. A plain `server.tick` subscription
/// runs every 20 server ticks; use [`subscribe_with_interval`] to choose a cadence.
///
/// Cancellable event handlers run synchronously on the owning game thread so their result can
/// affect the original NeoForge event. Use `#[event_async]` for non-cancellable notifications.
pub fn subscribe(event_name: &str, priority: EventPriority) -> SjmlResult<EventSubscriptionHandle> {
    validate_event_name(event_name)?;
    #[cfg(target_arch = "wasm32")]
    {
        subscribe_via_host(event_name, priority, None)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = priority;
        Err(unsupported_target())
    }
}

/// Registers a `server.tick` callback at the requested cadence (1–1,200 ticks).
pub fn subscribe_with_interval(
    event_name: &str,
    priority: EventPriority,
    interval_ticks: u32,
) -> SjmlResult<EventSubscriptionHandle> {
    validate_event_name(event_name)?;
    if event_name != "server.tick" {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "interval subscriptions are only supported for server.tick",
        ));
    }
    if !(1..=MAX_SERVER_TICK_INTERVAL).contains(&interval_ticks) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "server.tick interval must be between 1 and 1200 ticks",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        subscribe_via_host(event_name, priority, Some(interval_ticks))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = priority;
        Err(unsupported_target())
    }
}

#[cfg(target_arch = "wasm32")]
fn subscribe_via_host(
    event_name: &str,
    priority: EventPriority,
    interval_ticks: Option<u32>,
) -> SjmlResult<EventSubscriptionHandle> {
    const HANDLE_BYTES: u32 = 24;
    let output_pointer = allocate_host_input(HANDLE_BYTES);
    if output_pointer == 0 {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "could not allocate the event subscription handle output buffer",
        ));
    }
    // SAFETY: the allocator returned a live 24-byte buffer owned by this module instance.
    let result = unsafe {
        let event_pointer = event_name.as_ptr() as usize as u32 as i32;
        let event_length = event_name.len() as u32 as i32;
        let status = match interval_ticks {
            Some(interval) => imported_subscribe_interval(
                event_pointer,
                event_length,
                priority as i32,
                interval as i32,
                output_pointer as i32,
            ),
            None => imported_subscribe(
                event_pointer,
                event_length,
                priority as i32,
                output_pointer as i32,
            ),
        };
        if status != 0 {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected the event subscription",
            ))
        } else {
            let bytes =
                core::slice::from_raw_parts(output_pointer as *const u8, HANDLE_BYTES as usize);
            from_bytes(bytes)
        }
    };
    // SAFETY: the same live output allocation is released exactly once after the host call ends.
    unsafe { deallocate_host_input(output_pointer, HANDLE_BYTES) };
    result
}

/// Removes one subscription. Calling this again with the same copied handle returns a stale-handle error.
pub fn unsubscribe(subscription: EventSubscriptionHandle) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let bytes = to_bytes(&subscription)?;
        // SAFETY: the borrowed stable encoding stays live for the complete synchronous host call.
        let status = unsafe {
            imported_unsubscribe(
                bytes.as_ptr() as usize as u32 as i32,
                bytes.len() as u32 as i32,
            )
        };
        status_result(status, "host rejected the event unsubscription")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = subscription;
        Err(unsupported_target())
    }
}

fn validate_event_name(event_name: &str) -> SjmlResult<()> {
    if event_name.is_empty()
        || event_name.len() > 128
        || !matches!(event_name.as_bytes()[0], b'a'..=b'z' | b'0'..=b'9')
        || !event_name.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"._-".contains(&byte)
        })
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "event name must contain 1..=128 lowercase ASCII letters, digits, '.', '_' or '-'; it must start with a letter or digit",
        ));
    }
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn status_result(status: i32, message: &str) -> SjmlResult<()> {
    if status == 0 {
        Ok(())
    } else {
        Err(SjmlError::from_abi_code(status as u32, message))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn unsupported_target() -> SjmlError {
    SjmlError::new(
        ErrorCode::UnknownErrorCode,
        "event subscriptions require the wasm32-unknown-unknown target",
    )
}

#[cfg(test)]
mod tests {
    use super::{
        subscribe, subscribe_with_interval, unsubscribe, BlockBreak, BlockPlace, EffectTickEvent,
        EntityDamaged, EntityDeath, EntitySpawned, EventPriority, InventoryChanged,
        PlayerInteracted, ResourcesReloaded, ServerTick, WorldLifecycle, MAX_SERVER_TICK_INTERVAL,
    };
    use crate::inventory::BlockPos;
    use crate::ErrorCode;
    use crate::{from_bytes, to_bytes};

    #[test]
    fn block_break_snapshot_round_trips_signed_coordinates() {
        let event = BlockBreak {
            player_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            block_id: "minecraft:stone".to_owned(),
            x: -3,
            y: 64,
            z: i32::MIN,
        };

        let encoded = to_bytes(&event).unwrap();
        let decoded: BlockBreak = from_bytes(&encoded).unwrap();

        assert_eq!(decoded, event);
    }

    #[test]
    fn block_place_snapshot_round_trips_optional_placer_and_signed_coordinates() {
        let event = BlockPlace {
            placer_id: Some("00000000-0000-0000-0000-000000000001".to_owned()),
            dimension_id: "minecraft:overworld".to_owned(),
            block_id: "minecraft:diamond_block".to_owned(),
            x: -3,
            y: 64,
            z: i32::MIN,
        };
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(from_bytes::<BlockPlace>(&encoded).unwrap(), event);

        let no_placer = BlockPlace {
            placer_id: None,
            ..event
        };
        let encoded = to_bytes(&no_placer).unwrap();
        assert_eq!(from_bytes::<BlockPlace>(&encoded).unwrap(), no_placer);
    }

    #[test]
    fn entity_death_snapshot_round_trips_stable_identity_and_coordinates() {
        let event = EntityDeath {
            entity_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            entity_type_id: "minecraft:pig".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            x: -1.5,
            y: 64.0,
            z: 2.25,
        };
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(from_bytes::<EntityDeath>(&encoded).unwrap(), event);
    }

    #[test]
    fn entity_spawned_snapshot_round_trips_world_load_metadata() {
        let event = EntitySpawned {
            entity_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            entity_type_id: "minecraft:pig".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            x: -1.5,
            y: 64.0,
            z: 2.25,
            loaded_from_disk: true,
        };
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(from_bytes::<EntitySpawned>(&encoded).unwrap(), event);
    }

    #[test]
    fn effect_tick_snapshot_round_trips_registered_tick_metadata() {
        let event = EffectTickEvent {
            effect_id: "sjml_game_test:content/gametest_tick".to_owned(),
            entity_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            entity_type_id: "minecraft:pig".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            x: -1.5,
            y: 64.0,
            z: 2.25,
            amplifier: 2,
            duration_ticks: 100,
            tick_interval: 20,
        };
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(from_bytes::<EffectTickEvent>(&encoded).unwrap(), event);
    }

    #[test]
    fn player_interaction_snapshot_round_trips_optional_target_details() {
        let event = PlayerInteracted {
            player_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            action: "right_click_block".to_owned(),
            hand: "main_hand".to_owned(),
            target_id: Some("minecraft:stone".to_owned()),
            face: Some("up".to_owned()),
            block_position: Some(BlockPos {
                x: -3,
                y: 64,
                z: i32::MIN,
            }),
        };
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(from_bytes::<PlayerInteracted>(&encoded).unwrap(), event);

        let no_target = PlayerInteracted {
            target_id: None,
            face: None,
            block_position: None,
            ..event
        };
        let encoded = to_bytes(&no_target).unwrap();
        assert_eq!(from_bytes::<PlayerInteracted>(&encoded).unwrap(), no_target);
    }

    #[test]
    fn entity_damaged_snapshot_round_trips_optional_source_and_applied_damage() {
        let event = EntityDamaged {
            entity_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            entity_type_id: "minecraft:pig".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            damage_type_id: "minecraft:mob_attack".to_owned(),
            source_entity_id: Some("00000000-0000-0000-0000-000000000002".to_owned()),
            x: -1.5,
            y: 64.0,
            z: 2.25,
            original_damage: 5.0,
            applied_damage: 3.0,
        };
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(from_bytes::<EntityDamaged>(&encoded).unwrap(), event);
    }

    #[test]
    fn world_lifecycle_snapshot_round_trips_dimension_and_game_time() {
        let event = WorldLifecycle {
            dimension_id: "minecraft:the_nether".to_owned(),
            game_time: 1234,
        };
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(from_bytes::<WorldLifecycle>(&encoded).unwrap(), event);
    }

    #[test]
    fn server_tick_snapshot_round_trips_the_logical_tick_count() {
        let event = ServerTick { tick_count: 1234 };
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(from_bytes::<ServerTick>(&encoded).unwrap(), event);
    }

    #[test]
    fn resource_reload_snapshot_round_trips_its_committed_server_tick() {
        let event = ResourcesReloaded { tick_count: 1234 };
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(from_bytes::<ResourcesReloaded>(&encoded).unwrap(), event);
    }

    #[test]
    fn inventory_change_snapshot_round_trips_bounded_slot_and_count_data() {
        let event = InventoryChanged {
            player_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            item_id: "minecraft:diamond".to_owned(),
            change_type: "picked_up".to_owned(),
            slot: Some(35),
            count: 2,
        };
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(from_bytes::<InventoryChanged>(&encoded).unwrap(), event);

        let unknown_slot = InventoryChanged {
            slot: None,
            ..event
        };
        let encoded = to_bytes(&unknown_slot).unwrap();
        assert_eq!(
            from_bytes::<InventoryChanged>(&encoded).unwrap(),
            unknown_slot
        );
    }

    #[test]
    fn event_priority_values_match_the_stable_host_order() {
        assert_eq!(EventPriority::Highest as i32, 0);
        assert_eq!(EventPriority::High as i32, 1);
        assert_eq!(EventPriority::Normal as i32, 2);
        assert_eq!(EventPriority::Low as i32, 3);
        assert_eq!(EventPriority::Lowest as i32, 4);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_event_subscription_reports_unsupported_host_imports() {
        assert_eq!(
            subscribe("player.joined", EventPriority::Normal)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        let handle = crate::EventSubscriptionHandle::new(1, 2, 1, 1).unwrap();
        assert_eq!(
            unsubscribe(handle).unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn tick_subscription_interval_is_validated_before_host_access() {
        assert_eq!(
            subscribe_with_interval("player.joined", EventPriority::Normal, 20)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            subscribe_with_interval("server.tick", EventPriority::Normal, 0)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            subscribe_with_interval(
                "server.tick",
                EventPriority::Normal,
                MAX_SERVER_TICK_INTERVAL + 1
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            subscribe_with_interval("server.tick", EventPriority::Normal, 20)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn event_names_are_validated_before_host_access() {
        assert_eq!(
            subscribe("../player.joined", EventPriority::Normal)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            subscribe(&"a".repeat(129), EventPriority::Normal)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }
}
