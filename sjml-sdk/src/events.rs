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

/// Synchronous, non-cancellable snapshot emitted after meaningful logical-server player movement.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerMoved {
    /// Canonical UUID string of the player who moved.
    pub player_id: String,
    /// Current in-game profile name of the player.
    pub display_name: String,
    /// Resource location of the level where the player is currently located.
    pub dimension_id: String,
    /// Player X coordinate after movement was processed.
    pub x: f64,
    /// Player Y coordinate after movement was processed.
    pub y: f64,
    /// Player Z coordinate after movement was processed.
    pub z: f64,
    /// Player yaw in degrees after movement was processed.
    pub yaw: f32,
    /// Player pitch in degrees after movement was processed.
    pub pitch: f32,
}

/// Synchronous, non-cancellable snapshot emitted after a logical-server player respawn.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerRespawned {
    /// Canonical UUID string of the player who respawned.
    pub player_id: String,
    /// Current in-game profile name of the player.
    pub display_name: String,
    /// Resource location of the level where the player respawned.
    pub dimension_id: String,
    /// Player X coordinate after respawn placement.
    pub x: f64,
    /// Player Y coordinate after respawn placement.
    pub y: f64,
    /// Player Z coordinate after respawn placement.
    pub z: f64,
    /// Whether this respawn followed completion of the End credits.
    pub end_conquered: bool,
}

/// Synchronous, non-cancellable notification emitted after a server player earns an advancement.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerAdvancementEarned {
    /// Canonical UUID string of the player who earned the advancement.
    pub player_id: String,
    /// Current in-game profile name of the player.
    pub display_name: String,
    /// Resource location of the level where the advancement was earned.
    pub dimension_id: String,
    /// Registry ID of the earned advancement.
    pub advancement_id: String,
}

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

/// Synchronous cancellable logical-server right-click attempt for a block, entity, or item use.
///
/// Return `Ok(true)` from the `player.interaction` event handler to veto this attempt.
/// The host emits this event for right-click block/entity actions and right-click item use;
/// left-click actions remain outside this contract.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerInteractionAttempt {
    /// Canonical UUID string of the player who initiated the interaction.
    pub player_id: String,
    /// Resource location of the level where the interaction occurred.
    pub dimension_id: String,
    /// `right_click_block`, `entity_interact`, `entity_interact_specific`, or `right_click_item`.
    pub action: String,
    /// `main_hand` or `off_hand`.
    pub hand: String,
    /// Registry ID of the targeted block, entity type, or held item, when available.
    pub target_id: Option<String>,
    /// Block face for block interactions, when available.
    pub face: Option<String>,
    /// Target block position for block interactions, when available.
    pub block_position: Option<BlockPos>,
}

/// Synchronous cancellable attempt by a logical-server player to sleep at a bed.
///
/// NeoForge 21.1.265 exposes this through `CanPlayerSleepEvent`. Return `Ok(true)` from a
/// `player.sleep_attempt` handler to reject the attempt when `can_veto` is true. The host uses
/// NeoForge's sleep-problem result instead of event cancellation and preserves any denial already
/// set by vanilla or another mod. The callback runs on the server thread and requires both
/// `events.subscribe` and `events.cancel`.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerSleepAttempt {
    /// Canonical UUID string of the player attempting to sleep.
    pub player_id: String,
    /// Current in-game profile name of the player.
    pub player_name: String,
    /// Resource location of the level containing the bed.
    pub dimension_id: String,
    /// Integer position of the bed block.
    pub bed_position: BlockPos,
    /// Vanilla's original sleep problem, if any, as its stable enum name.
    pub vanilla_problem_id: Option<String>,
    /// Current sleep problem after higher-priority NeoForge listeners, if any.
    pub current_problem_id: Option<String>,
    /// Whether the host can still reject the attempt without replacing an existing denial.
    pub can_veto: bool,
}

/// Synchronous cancellable logical-server attempt to drop an item stack from a player inventory.
///
/// Return `Ok(true)` from the `player.drop_attempt` handler to veto the toss when `can_cancel` is
/// true. NeoForge posts `ItemTossEvent` after the stack leaves its slot, so the host restores the
/// stack before canceling. If the complete stack cannot fit in the player's inventory, the host
/// reports `can_cancel = false` and preserves vanilla dropping to prevent item loss.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerDropAttempt {
    /// Canonical UUID string of the player who attempted to drop the item.
    pub player_id: String,
    /// Resource location of the level where the toss was initiated.
    pub dimension_id: String,
    /// Registry ID of the item stack being tossed.
    pub item_id: String,
    /// Number of items in the tossed stack, bounded to `1..=99`.
    pub count: u32,
    /// Whether the whole stack can currently be restored to the player's inventory if vetoed.
    pub can_cancel: bool,
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

/// Synchronous cancellable damage attempt for a loaded living entity on the logical server.
///
/// This snapshot is delivered from NeoForge 21.1.265's `LivingIncomingDamageEvent`, after
/// invulnerability checks and before damage reductions such as armor are applied. Return
/// `Ok(true)` from the `entity.damage_attempt` handler to veto the entire incoming damage
/// sequence. The callback runs synchronously on the owning server thread and receives only
/// detached values.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityDamageAttempt {
    /// Canonical UUID string of the living entity that would receive damage.
    pub entity_id: String,
    /// Registry ID of the damaged entity type.
    pub entity_type_id: String,
    /// Resource location of the entity's level.
    pub dimension_id: String,
    /// Registry ID of the incoming damage type.
    pub damage_type_id: String,
    /// UUID of the responsible source entity, when the damage had one.
    pub source_entity_id: Option<String>,
    /// Entity X coordinate at the time the attempt was dispatched.
    pub x: f64,
    /// Entity Y coordinate at the time the attempt was dispatched.
    pub y: f64,
    /// Entity Z coordinate at the time the attempt was dispatched.
    pub z: f64,
    /// Damage value passed into the damage sequence before modifications.
    pub original_damage: f32,
    /// Current damage value before armor and other mitigation.
    pub amount: f32,
}

/// Synchronous cancellable attempt to heal a living entity on the logical server.
///
/// NeoForge 21.1.265 fires this from `LivingEntity::heal` before applying the amount. Return
/// `Ok(true)` from an `entity.heal_attempt` handler to prevent the healing. The callback receives
/// only detached entity identity, position, and bounded amount data; it runs on the owning server
/// thread and requires both `events.subscribe` and `events.cancel`.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityHealAttempt {
    /// Canonical UUID string of the living entity that would be healed.
    pub entity_id: String,
    /// Registry ID of the living entity type.
    pub entity_type_id: String,
    /// Resource location of the entity's level.
    pub dimension_id: String,
    /// Entity X coordinate at the time the attempt was dispatched.
    pub x: f64,
    /// Entity Y coordinate at the time the attempt was dispatched.
    pub y: f64,
    /// Entity Z coordinate at the time the attempt was dispatched.
    pub z: f64,
    /// Finite vanilla heal amount, bounded to `0..=1_000_000`.
    pub amount: f32,
}

/// Synchronous cancellable attempt to apply a status effect to a living entity.
///
/// NeoForge 21.1.265 fires this from `MobEffectEvent.Applicable` whenever it checks whether an
/// effect can be applied. Return `Ok(true)` from an `entity.effect_application` handler to set the
/// event result to `DO_NOT_APPLY`. The callback receives detached effect values and optional
/// source identity and runs on the owning logical-server thread. It requires both
/// `events.subscribe` and `events.cancel`.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityEffectApplicationAttempt {
    /// Canonical UUID string of the living entity that would receive the effect.
    pub entity_id: String,
    /// Registry ID of the affected living entity type.
    pub entity_type_id: String,
    /// Resource location of the affected entity's level.
    pub dimension_id: String,
    /// Registry ID of the status effect that would be applied.
    pub effect_id: String,
    /// UUID of the source entity, when NeoForge provides one.
    pub source_entity_id: Option<String>,
    /// Registry ID of the source entity type, when it is registered.
    pub source_entity_type_id: Option<String>,
    /// Affected entity X coordinate at the time of the applicability check.
    pub x: f64,
    /// Affected entity Y coordinate at the time of the applicability check.
    pub y: f64,
    /// Affected entity Z coordinate at the time of the applicability check.
    pub z: f64,
    /// Requested effect duration in ticks; `-1` represents an infinite duration.
    pub duration_ticks: i32,
    /// Zero-based effect amplifier.
    pub amplifier: i32,
}

/// Synchronous cancellable attempt to change a living entity's target on the logical server.
///
/// Return `Ok(true)` from the `entity.target_attempt` handler to keep the entity's existing
/// target. The event provides the requested target at the time SJML observes it; `None` means
/// NeoForge is attempting to clear the target. Target changes may come from a mob's explicit
/// target setter or its behavior-tree attack-target behavior.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityTargetAttempt {
    /// Canonical UUID string of the living entity whose target would change.
    pub entity_id: String,
    /// UUID of the requested new target, or `None` when attempting to clear the target.
    pub requested_target_id: Option<String>,
    /// Stable target-change origin, such as `LivingTargetType#MOB_TARGET` or
    /// `LivingTargetType#BEHAVIOR_TARGET`; mod-defined origins use their fully-qualified class name.
    pub target_change_type: String,
}

/// Synchronous, cancellable server-side projectile impact attempt.
///
/// Return `Ok(true)` from the `entity.projectile_impact_attempt` handler to cancel the impact; the
/// projectile then continues flying according to NeoForge's `ProjectileImpactEvent` contract.
/// The event is detached and identifies either an entity hit or a block hit. For block impacts,
/// the host reads the block ID only after confirming that the hit chunk is already loaded.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct ProjectileImpactAttempt {
    /// Canonical lowercase UUID of the projectile.
    pub projectile_id: String,
    /// Registry ID of the projectile entity type.
    pub projectile_type_id: String,
    /// Resource location of the level containing the projectile.
    pub dimension_id: String,
    /// `entity` or `block`.
    pub hit_kind: String,
    /// UUID of the hit entity for `entity` impacts; otherwise `None`.
    pub hit_entity_id: Option<String>,
    /// Registry ID of the hit entity type for `entity` impacts; otherwise `None`.
    pub hit_entity_type_id: Option<String>,
    /// Registry ID of the hit block for `block` impacts; otherwise `None`.
    pub hit_block_id: Option<String>,
    /// Block position for `block` impacts; its nested record schema is version 1.
    pub block_position: Option<BlockPos>,
    /// Face name such as `north` for `block` impacts; otherwise `None`.
    pub block_face: Option<String>,
    /// World X coordinate where the projectile ray hit.
    pub x: f64,
    /// World Y coordinate where the projectile ray hit.
    pub y: f64,
    /// World Z coordinate where the projectile ray hit.
    pub z: f64,
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

/// Synchronous cancellable server-side explosion start, before any damage or block changes apply.
///
/// Return `Ok(true)` from the `world.explosion_attempt` handler to stop the entire explosion.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct ExplosionAttempt {
    /// Resource location identifying the level that would be exploded.
    pub dimension_id: String,
    /// Direct source entity UUID when the explosion was caused by an entity.
    pub source_entity_id: Option<String>,
    /// Registry ID of the direct source entity type when present.
    pub source_entity_type_id: Option<String>,
    /// Vanilla block behavior: `keep`, `destroy`, `destroy_with_decay`, or `trigger_block`.
    pub block_interaction: String,
    /// Explosion center X coordinate.
    pub x: f64,
    /// Explosion center Y coordinate.
    pub y: f64,
    /// Explosion center Z coordinate.
    pub z: f64,
    /// Vanilla explosion radius, bounded to `0..=128`.
    pub radius: f32,
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
    /// Registry ID of the block type at the attempted position; state properties are omitted.
    pub block_id: String,
    /// Signed block X coordinate.
    pub x: i32,
    /// Signed block Y coordinate.
    pub y: i32,
    /// Signed block Z coordinate.
    pub z: i32,
}

/// Synchronous cancellable attempt to break a block on the logical server.
///
/// Return `Ok(true)` from a `#[event(name = "block.break_attempt")]` handler to
/// veto destruction. The callback runs before the block is removed on the
/// server thread and receives only detached values. A veto requires both
/// `events.subscribe` and `events.cancel`.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct BlockBreakAttempt {
    /// Canonical UUID string of the player who attempted to break the block.
    pub player_id: String,
    /// Resource location of the owning dimension, such as `minecraft:overworld`.
    pub dimension_id: String,
    /// Registry ID of the block type at the attempted position; state properties are omitted.
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

/// Synchronous cancellable snapshot for a logical-server block-placement attempt.
///
/// Return `Ok(true)` from the `block.place_attempt` handler to veto the placement.
/// For a multi-place event, the payload describes NeoForge's event position and a veto
/// rejects the full placement operation.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct BlockPlaceAttempt {
    /// UUID of the entity that attempted placement, if the event had an entity actor.
    pub placer_id: Option<String>,
    /// Resource location of the owning dimension.
    pub dimension_id: String,
    /// Registry ID of the block state that was proposed for placement.
    pub block_id: String,
    /// Signed block X coordinate for the NeoForge placement event.
    pub x: i32,
    /// Signed block Y coordinate for the NeoForge placement event.
    pub y: i32,
    /// Signed block Z coordinate for the NeoForge placement event.
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

/// Synchronous cancellable death attempt for a loaded living entity on the logical server.
///
/// Return `Ok(true)` from the `entity.death_attempt` handler to cancel this death. For player
/// victims, `player_display_name` contains their detached profile name; the UUID is `entity_id`.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityDeathAttempt {
    /// Canonical lowercase UUID of the living entity whose death is being processed.
    pub entity_id: String,
    /// Canonical registry ID of the entity type.
    pub entity_type_id: String,
    /// Canonical dimension ID where the death attempt fired.
    pub dimension_id: String,
    /// Player profile name for a player victim; `None` for all other entity types.
    pub player_display_name: Option<String>,
    /// Entity X coordinate when the attempt was dispatched.
    pub x: f64,
    /// Entity Y coordinate when the attempt was dispatched.
    pub y: f64,
    /// Entity Z coordinate when the attempt was dispatched.
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

/// Synchronous cancellable attempt for an entity to join a logical-server level.
///
/// NeoForge fires `EntityJoinLevelEvent` on both logical sides before insertion. The SJML host
/// only dispatches the server-side event and skips level chunk queries in this pre-add hook.
/// Return `Ok(true)` from an `entity.join_attempt` handler to cancel insertion. The callback
/// receives only detached identity, dimension, position, and load-origin values and requires both
/// `events.subscribe` and `events.cancel`.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityJoinAttempt {
    /// Canonical lowercase UUID of the entity that would join the level.
    pub entity_id: String,
    /// Registry ID of the entity type.
    pub entity_type_id: String,
    /// Canonical dimension ID of the target level.
    pub dimension_id: String,
    /// Entity X coordinate before insertion.
    pub x: f64,
    /// Entity Y coordinate before insertion.
    pub y: f64,
    /// Entity Z coordinate before insertion.
    pub z: f64,
    /// Whether the entity is being restored from saved world data.
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
        subscribe, subscribe_with_interval, unsubscribe, BlockBreak, BlockBreakAttempt, BlockPlace,
        BlockPlaceAttempt, EffectTickEvent, EntityDamageAttempt, EntityDamaged, EntityDeath,
        EntityDeathAttempt, EntityEffectApplicationAttempt, EntityHealAttempt, EntityJoinAttempt,
        EntitySpawned, EntityTargetAttempt, EventPriority, ExplosionAttempt, InventoryChanged,
        PlayerAdvancementEarned, PlayerDropAttempt, PlayerInteracted, PlayerInteractionAttempt,
        PlayerMoved, PlayerRespawned, PlayerSleepAttempt, ProjectileImpactAttempt,
        ResourcesReloaded, ServerTick, WorldLifecycle, MAX_SERVER_TICK_INTERVAL,
    };
    use crate::inventory::BlockPos;
    use crate::ErrorCode;
    use crate::{from_bytes, to_bytes};

    #[test]
    fn player_moved_snapshot_round_trips_detached_position_and_look() {
        let event = PlayerMoved {
            player_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            display_name: "Alice".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            x: -1.5,
            y: 64.0,
            z: 2.25,
            yaw: 90.0,
            pitch: -15.5,
        };

        let encoded = to_bytes(&event).unwrap();
        assert_eq!(from_bytes::<PlayerMoved>(&encoded).unwrap(), event);
    }

    #[test]
    fn player_respawned_snapshot_round_trips_identity_position_and_end_state() {
        let event = PlayerRespawned {
            player_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            display_name: "Alice".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            x: -1.5,
            y: 64.0,
            z: 2.25,
            end_conquered: true,
        };

        let encoded = to_bytes(&event).unwrap();
        assert_eq!(from_bytes::<PlayerRespawned>(&encoded).unwrap(), event);
    }

    #[test]
    fn advancement_earned_snapshot_round_trips_player_and_registry_identity() {
        let event = PlayerAdvancementEarned {
            player_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            display_name: "Alice".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            advancement_id: "minecraft:story/root".to_owned(),
        };

        let encoded = to_bytes(&event).unwrap();
        assert_eq!(
            from_bytes::<PlayerAdvancementEarned>(&encoded).unwrap(),
            event
        );
    }

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
    fn block_break_attempt_round_trips_detached_player_block_and_signed_coordinates() {
        let attempt = BlockBreakAttempt {
            player_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            block_id: "minecraft:stone".to_owned(),
            x: -3,
            y: 320,
            z: i32::MIN,
        };

        let encoded = to_bytes(&attempt).unwrap();
        let decoded: BlockBreakAttempt = from_bytes(&encoded).unwrap();

        assert_eq!(decoded, attempt);
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
    fn cancellable_block_place_attempt_round_trips_optional_placer_and_coordinates() {
        let attempt = BlockPlaceAttempt {
            placer_id: Some("00000000-0000-0000-0000-000000000001".to_owned()),
            dimension_id: "minecraft:overworld".to_owned(),
            block_id: "minecraft:diamond_block".to_owned(),
            x: -3,
            y: 64,
            z: i32::MIN,
        };
        let encoded = to_bytes(&attempt).unwrap();
        assert_eq!(from_bytes::<BlockPlaceAttempt>(&encoded).unwrap(), attempt);

        let no_placer = BlockPlaceAttempt {
            placer_id: None,
            ..attempt
        };
        let encoded = to_bytes(&no_placer).unwrap();
        assert_eq!(
            from_bytes::<BlockPlaceAttempt>(&encoded).unwrap(),
            no_placer
        );
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
    fn cancellable_entity_death_attempt_round_trips_player_identity_and_coordinates() {
        let event = EntityDeathAttempt {
            entity_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            entity_type_id: "minecraft:player".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            player_display_name: Some("Alice".to_owned()),
            x: -1.5,
            y: 64.0,
            z: 2.25,
        };
        let encoded = to_bytes(&event).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &event.entity_id);
        append_test_string(&mut expected, &event.entity_type_id);
        append_test_string(&mut expected, &event.dimension_id);
        expected.push(1);
        append_test_string(&mut expected, "Alice");
        expected.extend_from_slice(&(-1.5_f64).to_le_bytes());
        expected.extend_from_slice(&(64.0_f64).to_le_bytes());
        expected.extend_from_slice(&(2.25_f64).to_le_bytes());
        assert_eq!(
            encoded, expected,
            "Rust encoding must match the host schema-v1 bytes"
        );
        assert_eq!(from_bytes::<EntityDeathAttempt>(&encoded).unwrap(), event);

        let non_player = EntityDeathAttempt {
            player_display_name: None,
            entity_type_id: "minecraft:pig".to_owned(),
            x: 1.0,
            y: 2.0,
            z: 3.0,
            ..event
        };
        let encoded = to_bytes(&non_player).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &non_player.entity_id);
        append_test_string(&mut expected, &non_player.entity_type_id);
        append_test_string(&mut expected, &non_player.dimension_id);
        expected.push(0);
        expected.extend_from_slice(&(1.0_f64).to_le_bytes());
        expected.extend_from_slice(&(2.0_f64).to_le_bytes());
        expected.extend_from_slice(&(3.0_f64).to_le_bytes());
        assert_eq!(
            encoded, expected,
            "optional player data must use the stable None tag"
        );
        assert_eq!(
            from_bytes::<EntityDeathAttempt>(&encoded).unwrap(),
            non_player
        );
    }

    #[test]
    fn cancellable_entity_damage_attempt_matches_host_schema_v1_bytes() {
        let event = EntityDamageAttempt {
            entity_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            entity_type_id: "minecraft:pig".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            damage_type_id: "minecraft:player_attack".to_owned(),
            source_entity_id: Some("00000000-0000-0000-0000-000000000002".to_owned()),
            x: -1.5,
            y: 64.0,
            z: 2.25,
            original_damage: 8.0,
            amount: 8.0,
        };
        let encoded = to_bytes(&event).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &event.entity_id);
        append_test_string(&mut expected, &event.entity_type_id);
        append_test_string(&mut expected, &event.dimension_id);
        append_test_string(&mut expected, &event.damage_type_id);
        expected.push(1);
        append_test_string(&mut expected, event.source_entity_id.as_deref().unwrap());
        expected.extend_from_slice(&(-1.5_f64).to_le_bytes());
        expected.extend_from_slice(&(64.0_f64).to_le_bytes());
        expected.extend_from_slice(&(2.25_f64).to_le_bytes());
        expected.extend_from_slice(&(8.0_f32).to_le_bytes());
        expected.extend_from_slice(&(8.0_f32).to_le_bytes());
        assert_eq!(
            encoded, expected,
            "Rust encoding must match the host schema-v1 bytes"
        );
        assert_eq!(from_bytes::<EntityDamageAttempt>(&encoded).unwrap(), event);

        let environment_damage = EntityDamageAttempt {
            source_entity_id: None,
            damage_type_id: "minecraft:generic".to_owned(),
            x: 1.0,
            y: 2.0,
            z: 3.0,
            original_damage: 2.0,
            amount: 1.5,
            ..event
        };
        let encoded = to_bytes(&environment_damage).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &environment_damage.entity_id);
        append_test_string(&mut expected, &environment_damage.entity_type_id);
        append_test_string(&mut expected, &environment_damage.dimension_id);
        append_test_string(&mut expected, &environment_damage.damage_type_id);
        expected.push(0);
        expected.extend_from_slice(&(1.0_f64).to_le_bytes());
        expected.extend_from_slice(&(2.0_f64).to_le_bytes());
        expected.extend_from_slice(&(3.0_f64).to_le_bytes());
        expected.extend_from_slice(&(2.0_f32).to_le_bytes());
        expected.extend_from_slice(&(1.5_f32).to_le_bytes());
        assert_eq!(
            encoded, expected,
            "None source entity uses the stable option tag"
        );
        assert_eq!(
            from_bytes::<EntityDamageAttempt>(&encoded).unwrap(),
            environment_damage
        );
    }

    #[test]
    fn cancellable_entity_heal_attempt_matches_host_schema_v1_bytes() {
        let event = EntityHealAttempt {
            entity_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            entity_type_id: "minecraft:pig".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            x: -1.5,
            y: 64.0,
            z: 2.25,
            amount: 4.0,
        };
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(from_bytes::<EntityHealAttempt>(&encoded).unwrap(), event);
    }

    #[test]
    fn cancellable_entity_effect_application_matches_host_schema_v1_bytes() {
        let event = EntityEffectApplicationAttempt {
            entity_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            entity_type_id: "minecraft:cow".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            effect_id: "minecraft:poison".to_owned(),
            source_entity_id: Some("00000000-0000-0000-0000-000000000002".to_owned()),
            source_entity_type_id: Some("minecraft:pig".to_owned()),
            x: -1.5,
            y: 64.0,
            z: 2.25,
            duration_ticks: 1234,
            amplifier: 2,
        };
        let encoded = to_bytes(&event).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &event.entity_id);
        append_test_string(&mut expected, &event.entity_type_id);
        append_test_string(&mut expected, &event.dimension_id);
        append_test_string(&mut expected, &event.effect_id);
        expected.push(1);
        append_test_string(&mut expected, event.source_entity_id.as_deref().unwrap());
        expected.push(1);
        append_test_string(
            &mut expected,
            event.source_entity_type_id.as_deref().unwrap(),
        );
        expected.extend_from_slice(&(-1.5_f64).to_le_bytes());
        expected.extend_from_slice(&(64.0_f64).to_le_bytes());
        expected.extend_from_slice(&(2.25_f64).to_le_bytes());
        expected.extend_from_slice(&1234_i32.to_le_bytes());
        expected.extend_from_slice(&2_i32.to_le_bytes());
        assert_eq!(
            encoded, expected,
            "Rust encoding must match the host schema-v1 bytes"
        );
        assert_eq!(
            from_bytes::<EntityEffectApplicationAttempt>(&encoded).unwrap(),
            event
        );

        let environment_effect = EntityEffectApplicationAttempt {
            source_entity_id: None,
            source_entity_type_id: None,
            ..event
        };
        let encoded = to_bytes(&environment_effect).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &environment_effect.entity_id);
        append_test_string(&mut expected, &environment_effect.entity_type_id);
        append_test_string(&mut expected, &environment_effect.dimension_id);
        append_test_string(&mut expected, &environment_effect.effect_id);
        expected.push(0);
        expected.push(0);
        expected.extend_from_slice(&(-1.5_f64).to_le_bytes());
        expected.extend_from_slice(&(64.0_f64).to_le_bytes());
        expected.extend_from_slice(&(2.25_f64).to_le_bytes());
        expected.extend_from_slice(&1234_i32.to_le_bytes());
        expected.extend_from_slice(&2_i32.to_le_bytes());
        assert_eq!(encoded, expected, "missing source uses stable option tags");
        assert_eq!(
            from_bytes::<EntityEffectApplicationAttempt>(&encoded).unwrap(),
            environment_effect
        );
    }

    #[test]
    fn cancellable_entity_target_attempt_matches_host_schema_v1_bytes() {
        let event = EntityTargetAttempt {
            entity_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            requested_target_id: Some("00000000-0000-0000-0000-000000000002".to_owned()),
            target_change_type: "net.neoforged.neoforge.event.entity.living.LivingChangeTargetEvent$LivingTargetType#MOB_TARGET".to_owned(),
        };
        let encoded = to_bytes(&event).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &event.entity_id);
        expected.push(1);
        append_test_string(&mut expected, event.requested_target_id.as_deref().unwrap());
        append_test_string(&mut expected, &event.target_change_type);
        assert_eq!(
            encoded, expected,
            "Rust encoding must match the host schema-v1 bytes"
        );
        assert_eq!(from_bytes::<EntityTargetAttempt>(&encoded).unwrap(), event);

        let clear_target = EntityTargetAttempt {
            requested_target_id: None,
            ..event
        };
        let encoded = to_bytes(&clear_target).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &clear_target.entity_id);
        expected.push(0);
        append_test_string(&mut expected, &clear_target.target_change_type);
        assert_eq!(encoded, expected, "None target uses the stable option tag");
        assert_eq!(
            from_bytes::<EntityTargetAttempt>(&encoded).unwrap(),
            clear_target
        );
    }

    #[test]
    fn cancellable_projectile_impact_attempt_matches_host_schema_v1_for_both_hit_kinds() {
        let entity_impact = ProjectileImpactAttempt {
            projectile_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            projectile_type_id: "minecraft:arrow".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            hit_kind: "entity".to_owned(),
            hit_entity_id: Some("00000000-0000-0000-0000-000000000002".to_owned()),
            hit_entity_type_id: Some("minecraft:cow".to_owned()),
            hit_block_id: None,
            block_position: None,
            block_face: None,
            x: -1.5,
            y: 64.0,
            z: 2.25,
        };
        let encoded = to_bytes(&entity_impact).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &entity_impact.projectile_id);
        append_test_string(&mut expected, &entity_impact.projectile_type_id);
        append_test_string(&mut expected, &entity_impact.dimension_id);
        append_test_string(&mut expected, &entity_impact.hit_kind);
        expected.push(1);
        append_test_string(
            &mut expected,
            entity_impact.hit_entity_id.as_deref().unwrap(),
        );
        expected.push(1);
        append_test_string(
            &mut expected,
            entity_impact.hit_entity_type_id.as_deref().unwrap(),
        );
        expected.extend_from_slice(&[0, 0, 0]);
        expected.extend_from_slice(&(-1.5_f64).to_le_bytes());
        expected.extend_from_slice(&(64.0_f64).to_le_bytes());
        expected.extend_from_slice(&(2.25_f64).to_le_bytes());
        assert_eq!(
            encoded, expected,
            "Rust encoding must match host schema-v1 bytes"
        );
        assert_eq!(
            from_bytes::<ProjectileImpactAttempt>(&encoded).unwrap(),
            entity_impact
        );

        let block_impact = ProjectileImpactAttempt {
            hit_kind: "block".to_owned(),
            hit_entity_id: None,
            hit_entity_type_id: None,
            hit_block_id: Some("minecraft:stone".to_owned()),
            block_position: Some(BlockPos { x: -3, y: 64, z: 7 }),
            block_face: Some("up".to_owned()),
            ..entity_impact
        };
        let encoded = to_bytes(&block_impact).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &block_impact.projectile_id);
        append_test_string(&mut expected, &block_impact.projectile_type_id);
        append_test_string(&mut expected, &block_impact.dimension_id);
        append_test_string(&mut expected, &block_impact.hit_kind);
        expected.extend_from_slice(&[0, 0]);
        expected.push(1);
        append_test_string(&mut expected, block_impact.hit_block_id.as_deref().unwrap());
        expected.push(1);
        expected.extend_from_slice(&1_u32.to_le_bytes()); // nested BlockPos schema version
        expected.extend_from_slice(&(-3_i32).to_le_bytes());
        expected.extend_from_slice(&(64_i32).to_le_bytes());
        expected.extend_from_slice(&(7_i32).to_le_bytes());
        expected.push(1);
        append_test_string(&mut expected, block_impact.block_face.as_deref().unwrap());
        expected.extend_from_slice(&(-1.5_f64).to_le_bytes());
        expected.extend_from_slice(&(64.0_f64).to_le_bytes());
        expected.extend_from_slice(&(2.25_f64).to_le_bytes());
        assert_eq!(
            encoded, expected,
            "Rust encoding must match host schema-v1 bytes"
        );
        assert_eq!(
            from_bytes::<ProjectileImpactAttempt>(&encoded).unwrap(),
            block_impact
        );
    }

    #[test]
    fn cancellable_explosion_attempt_round_trips_source_and_block_behavior() {
        let sourced = ExplosionAttempt {
            dimension_id: "minecraft:overworld".to_owned(),
            source_entity_id: Some("00000000-0000-0000-0000-000000000001".to_owned()),
            source_entity_type_id: Some("minecraft:tnt".to_owned()),
            block_interaction: "destroy_with_decay".to_owned(),
            x: -1.5,
            y: 64.0,
            z: 2.25,
            radius: 4.0,
        };
        let encoded = to_bytes(&sourced).unwrap();
        assert_eq!(from_bytes::<ExplosionAttempt>(&encoded).unwrap(), sourced);

        let source_free = ExplosionAttempt {
            source_entity_id: None,
            source_entity_type_id: None,
            ..sourced
        };
        let encoded = to_bytes(&source_free).unwrap();
        assert_eq!(
            from_bytes::<ExplosionAttempt>(&encoded).unwrap(),
            source_free
        );
    }

    fn append_test_string(output: &mut Vec<u8>, value: &str) {
        output.extend_from_slice(&(value.len() as i32).to_le_bytes());
        output.extend_from_slice(value.as_bytes());
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
    fn cancellable_entity_join_attempt_matches_host_schema_v1_bytes() {
        let event = EntityJoinAttempt {
            entity_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            entity_type_id: "minecraft:cow".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            x: -1.5,
            y: 64.0,
            z: 2.25,
            loaded_from_disk: true,
        };
        let encoded = to_bytes(&event).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &event.entity_id);
        append_test_string(&mut expected, &event.entity_type_id);
        append_test_string(&mut expected, &event.dimension_id);
        expected.extend_from_slice(&(-1.5_f64).to_le_bytes());
        expected.extend_from_slice(&(64.0_f64).to_le_bytes());
        expected.extend_from_slice(&(2.25_f64).to_le_bytes());
        expected.push(1);
        assert_eq!(
            encoded, expected,
            "Rust encoding must match the host schema-v1 bytes"
        );
        assert_eq!(from_bytes::<EntityJoinAttempt>(&encoded).unwrap(), event);

        let fresh_entity = EntityJoinAttempt {
            loaded_from_disk: false,
            ..event
        };
        let encoded = to_bytes(&fresh_entity).unwrap();
        assert_eq!(encoded.last(), Some(&0));
        assert_eq!(
            from_bytes::<EntityJoinAttempt>(&encoded).unwrap(),
            fresh_entity
        );
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
    fn cancellable_player_interaction_attempt_round_trips_optional_target_details() {
        let attempt = PlayerInteractionAttempt {
            player_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            action: "right_click_block".to_owned(),
            hand: "main_hand".to_owned(),
            target_id: Some("minecraft:lever".to_owned()),
            face: Some("up".to_owned()),
            block_position: Some(BlockPos { x: -2, y: 64, z: 7 }),
        };

        let encoded = to_bytes(&attempt).unwrap();
        assert_eq!(
            from_bytes::<PlayerInteractionAttempt>(&encoded).unwrap(),
            attempt
        );

        let entity_attempt = PlayerInteractionAttempt {
            action: "entity_interact".to_owned(),
            target_id: Some("minecraft:villager".to_owned()),
            face: None,
            block_position: None,
            ..attempt
        };
        let encoded = to_bytes(&entity_attempt).unwrap();
        assert_eq!(
            from_bytes::<PlayerInteractionAttempt>(&encoded).unwrap(),
            entity_attempt
        );

        let item_attempt = PlayerInteractionAttempt {
            action: "right_click_item".to_owned(),
            target_id: Some("minecraft:apple".to_owned()),
            face: None,
            block_position: None,
            ..entity_attempt
        };
        let encoded = to_bytes(&item_attempt).unwrap();
        assert_eq!(
            from_bytes::<PlayerInteractionAttempt>(&encoded).unwrap(),
            item_attempt
        );
    }

    #[test]
    fn cancellable_player_sleep_attempt_round_trips_bed_and_existing_problem_state() {
        let attempt = PlayerSleepAttempt {
            player_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            player_name: "Alice".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            bed_position: BlockPos { x: -2, y: 64, z: 7 },
            vanilla_problem_id: Some("NOT_POSSIBLE_NOW".to_owned()),
            current_problem_id: None,
            can_veto: true,
        };
        let encoded = to_bytes(&attempt).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &attempt.player_id);
        append_test_string(&mut expected, &attempt.player_name);
        append_test_string(&mut expected, &attempt.dimension_id);
        expected.extend_from_slice(&1_i32.to_le_bytes()); // nested BlockPos schema version
        expected.extend_from_slice(&(-2_i32).to_le_bytes());
        expected.extend_from_slice(&(64_i32).to_le_bytes());
        expected.extend_from_slice(&(7_i32).to_le_bytes());
        expected.push(1);
        append_test_string(&mut expected, "NOT_POSSIBLE_NOW");
        expected.push(0);
        expected.push(1);
        assert_eq!(
            encoded, expected,
            "Rust encoding must match the host schema-v1 bytes"
        );
        assert_eq!(from_bytes::<PlayerSleepAttempt>(&encoded).unwrap(), attempt);

        let already_denied = PlayerSleepAttempt {
            vanilla_problem_id: None,
            current_problem_id: Some("OTHER_PROBLEM".to_owned()),
            can_veto: false,
            ..attempt
        };
        let encoded = to_bytes(&already_denied).unwrap();
        assert_eq!(
            from_bytes::<PlayerSleepAttempt>(&encoded).unwrap(),
            already_denied
        );
    }

    #[test]
    fn player_drop_attempt_round_trips_item_stack_and_inventory_capacity() {
        let attempt = PlayerDropAttempt {
            player_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            item_id: "minecraft:diamond".to_owned(),
            count: 12,
            can_cancel: true,
        };
        let encoded = to_bytes(&attempt).unwrap();
        assert_eq!(from_bytes::<PlayerDropAttempt>(&encoded).unwrap(), attempt);

        let not_cancellable = PlayerDropAttempt {
            can_cancel: false,
            ..attempt
        };
        let encoded = to_bytes(&not_cancellable).unwrap();
        assert_eq!(encoded.last(), Some(&0));
        assert_eq!(
            from_bytes::<PlayerDropAttempt>(&encoded).unwrap(),
            not_cancellable
        );
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
