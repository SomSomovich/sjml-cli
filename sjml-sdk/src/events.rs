//! Stable payload records used by the built-in NeoForge event adapters.

use crate::abi::EventSubscriptionHandle;
#[cfg(target_arch = "wasm32")]
use crate::abi::{allocate_host_input, deallocate_host_input};
use crate::inventory::BlockPos;
#[cfg(target_arch = "wasm32")]
use crate::{from_bytes, to_bytes};
use crate::{Decoder, Encoder, ErrorCode, SjmlError, SjmlResult, StableDecode, StableEncode};

/// Default server, player, entity, and client tick subscription cadence, in ticks.
pub const DEFAULT_SERVER_TICK_INTERVAL: u32 = 20;
/// Maximum supported server and client tick subscription cadence, in ticks.
pub const MAX_SERVER_TICK_INTERVAL: u32 = 1_200;
/// Maximum item count represented by one `inventory.changed` notification.
pub const MAX_INVENTORY_CHANGE_COUNT: i32 = 4_096;
/// Maximum player inventory slot index represented by `inventory.changed`.
pub const MAX_INVENTORY_CHANGE_SLOT: i32 = 255;
/// Maximum output stack count represented by one `player.item_crafted` event.
pub const MAX_CRAFTED_ITEM_COUNT: i32 = 1_000_000;
/// Maximum detached item snapshots included in one `player.fished_attempt` callback.
pub const MAX_FISHED_DROPS: usize = 16;
/// Maximum count represented by one detached `player.fished_attempt` drop.
pub const MAX_FISHED_ITEM_COUNT: u32 = 1_000_000;
/// Maximum detached computed item drops included in one `block.drops_attempt` callback.
pub const MAX_BLOCK_DROPS_ATTEMPT_DROPS: usize = 32;
/// Maximum count represented by one detached `block.drops_attempt` item snapshot.
pub const MAX_BLOCK_DROPS_ATTEMPT_ITEM_COUNT: u32 = 1_000_000;
/// Maximum durability value represented by one detached `block.drops_attempt` item.
pub const MAX_BLOCK_DROPS_ATTEMPT_DURABILITY: u32 = 1_000_000;
/// Maximum experience amount represented by one `block.drops_attempt` event.
pub const MAX_BLOCK_DROPS_ATTEMPT_EXPERIENCE: u32 = 1_000_000;
/// Maximum serialized payload size for one `block.drops_attempt` callback.
pub const MAX_BLOCK_DROPS_ATTEMPT_PAYLOAD_BYTES: usize = 16_384;
/// Maximum callbacks invoked for one computed block-drop event.
pub const MAX_BLOCK_DROPS_ATTEMPT_CALLBACKS: usize = 64;
/// Separate grant required to veto `client.action_attempt` actions.
pub const CLIENT_ACTION_CANCEL_CAPABILITY: &str = "client.input.action_cancel";
/// Separate grant required to observe player advancement criterion progress.
pub const PLAYER_ADVANCEMENT_OBSERVE_CAPABILITY: &str = "player.advancement.observe";
/// Separate grant required to observe player equipment changes.
pub const PLAYER_EQUIPMENT_OBSERVE_CAPABILITY: &str = "player.equipment.observe";
/// Maximum count represented by one detached `player.equipment_changed` item snapshot.
pub const MAX_PLAYER_EQUIPMENT_ITEM_COUNT: u32 = 1_000_000;
/// Maximum callbacks invoked for one physical-client action attempt.
pub const MAX_CLIENT_ACTION_ATTEMPT_CALLBACKS: usize = 64;
/// Minimum XP delta accepted from `PlayerExperienceChangeAction::SetAmount`.
pub const MIN_PLAYER_EXPERIENCE_CHANGE_AMOUNT: i32 = -1_000_000;
/// Maximum XP delta accepted from `PlayerExperienceChangeAction::SetAmount`.
pub const MAX_PLAYER_EXPERIENCE_CHANGE_AMOUNT: i32 = 1_000_000;
/// Maximum level represented by a detached `player.experience_change_attempt` snapshot.
pub const MAX_PLAYER_EXPERIENCE_LEVEL: i32 = 20_000;
/// Minimum level delta accepted by `PlayerExperienceLevelChangeAction::SetLevels`.
pub const MIN_PLAYER_EXPERIENCE_LEVEL_DELTA: i32 = -20_000;
/// Maximum level delta accepted by `PlayerExperienceLevelChangeAction::SetLevels`.
pub const MAX_PLAYER_EXPERIENCE_LEVEL_DELTA: i32 = 20_000;
/// Maximum `level_before` value represented by the level-change event snapshot.
pub const MAX_PLAYER_EXPERIENCE_LEVEL_SNAPSHOT: i32 = 1_000_000;
/// Maximum count captured for one potion-use start callback.
pub const MAX_PLAYER_POTION_USE_ITEM_COUNT: u32 = 64;
/// Maximum use duration captured for one potion-use start callback, in ticks.
pub const MAX_PLAYER_POTION_USE_DURATION_TICKS: u32 = 72_000;
/// Maximum callbacks invoked for one potion-use attempt.
pub const MAX_PLAYER_POTION_USE_ATTEMPT_CALLBACKS: usize = 64;
/// Maximum count represented by one detached `player.totem_use_attempt` snapshot.
pub const MAX_PLAYER_TOTEM_USE_COUNT: u32 = 64;
/// Maximum callbacks invoked for one synchronous totem-use attempt.
pub const MAX_PLAYER_TOTEM_USE_ATTEMPT_CALLBACKS: usize = 64;

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

/// Criterion-level progress change reported after Minecraft updates advancement progress.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerAdvancementProgressEvent {
    /// Canonical UUID string of the player whose criterion changed.
    pub player_id: String,
    /// Canonical registry ID of the advancement.
    pub advancement_id: String,
    /// Advancement criterion name that was granted or revoked.
    pub criterion_id: String,
    /// Whether this transition granted or revoked the criterion.
    pub operation: AdvancementProgressOperation,
    /// Whether the advancement was complete after this criterion changed.
    pub is_done: bool,
}

/// Progress operation that produced a [`PlayerAdvancementProgressEvent`] notification.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AdvancementProgressOperation {
    /// The criterion became complete.
    Grant,
    /// The criterion was revoked.
    Revoke,
}

impl StableEncode for AdvancementProgressOperation {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        encoder.write_u32(match self {
            Self::Grant => 0,
            Self::Revoke => 1,
        })
    }
}

impl StableDecode for AdvancementProgressOperation {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        match decoder.read_u32()? {
            0 => Ok(Self::Grant),
            1 => Ok(Self::Revoke),
            value => Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                format!("advancement progress operation code {value} is not supported"),
            )),
        }
    }
}

impl StableEncode for PlayerAdvancementProgressEvent {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        validate_advancement_progress(self)?;
        encoder.write_u32(1)?;
        encode_bounded_event_string(&self.player_id, encoder)?;
        encode_bounded_event_string(&self.advancement_id, encoder)?;
        encode_bounded_event_string(&self.criterion_id, encoder)?;
        self.operation.encode(encoder)?;
        encoder.write_u8(u8::from(self.is_done))
    }
}

impl StableDecode for PlayerAdvancementProgressEvent {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        let version = decoder.read_u32()?;
        if version != 1 {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                format!("PlayerAdvancementProgressEvent schema version {version} is unsupported"),
            ));
        }
        let event = Self {
            player_id: decode_bounded_event_string(decoder)?,
            advancement_id: decode_bounded_event_string(decoder)?,
            criterion_id: decode_bounded_event_string(decoder)?,
            operation: AdvancementProgressOperation::decode(decoder)?,
            is_done: match decoder.read_u8()? {
                0 => false,
                1 => true,
                value => {
                    return Err(SjmlError::new(
                        ErrorCode::InvalidBoolean,
                        format!("boolean wire value must be 0 or 1, found {value}"),
                    ));
                }
            },
        };
        validate_advancement_progress(&event)?;
        Ok(event)
    }
}

fn validate_advancement_progress(event: &PlayerAdvancementProgressEvent) -> SjmlResult<()> {
    let invalid = || {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "advancement progress event contains an invalid identity or empty criterion",
        )
    };
    if !canonical_uuid(&event.player_id)
        || !canonical_resource_location(&event.advancement_id)
        || event.criterion_id.is_empty()
    {
        return Err(invalid());
    }
    if event.player_id.len() > 256
        || event.advancement_id.len() > 256
        || event.criterion_id.len() > 256
    {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "advancement progress event string exceeds the 256-byte limit",
        ));
    }
    Ok(())
}

/// Non-cancellable equipment transition emitted from NeoForge's server-side living-entity tick hook.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerEquipmentChanged {
    /// Canonical UUID string of the player whose equipment changed.
    pub player_id: String,
    /// Vanilla equipment slot that changed.
    pub slot: PlayerEquipmentSlot,
    /// Bounded item description before the change; `None` means the slot was empty.
    pub previous: Option<PlayerEquipmentItemSnapshot>,
    /// Bounded item description after the change; `None` means the slot is now empty.
    pub current: Option<PlayerEquipmentItemSnapshot>,
}

/// Vanilla player equipment slots supported by the schema-v1 notification.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PlayerEquipmentSlot {
    MainHand,
    OffHand,
    Feet,
    Legs,
    Chest,
    Head,
    Body,
}

impl PlayerEquipmentSlot {
    fn code(self) -> u32 {
        match self {
            Self::MainHand => 0,
            Self::OffHand => 1,
            Self::Feet => 2,
            Self::Legs => 3,
            Self::Chest => 4,
            Self::Head => 5,
            Self::Body => 6,
        }
    }

    fn from_code(value: u32) -> SjmlResult<Self> {
        match value {
            0 => Ok(Self::MainHand),
            1 => Ok(Self::OffHand),
            2 => Ok(Self::Feet),
            3 => Ok(Self::Legs),
            4 => Ok(Self::Chest),
            5 => Ok(Self::Head),
            6 => Ok(Self::Body),
            _ => Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                format!("player equipment slot code {value} is not supported"),
            )),
        }
    }
}

/// Bounded registry and durability fields for one equipped stack.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerEquipmentItemSnapshot {
    /// Canonical registry ID of the item.
    pub item_id: String,
    /// Stack count, bounded to one million.
    pub count: u32,
    /// Current durability damage (zero for non-damageable items).
    pub damage: u32,
    /// Maximum durability (zero for non-damageable items).
    pub max_damage: u32,
}

impl StableEncode for PlayerEquipmentChanged {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        validate_player_equipment_changed(self)?;
        encoder.write_u32(1)?;
        encode_bounded_event_string(&self.player_id, encoder)?;
        encoder.write_u32(self.slot.code())?;
        encode_equipment_item_option(&self.previous, encoder)?;
        encode_equipment_item_option(&self.current, encoder)
    }
}

impl StableDecode for PlayerEquipmentChanged {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        let version = decoder.read_u32()?;
        if version != 1 {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                format!("PlayerEquipmentChanged schema version {version} is unsupported"),
            ));
        }
        let event = Self {
            player_id: decode_bounded_event_string(decoder)?,
            slot: PlayerEquipmentSlot::from_code(decoder.read_u32()?)?,
            previous: decode_equipment_item_option(decoder)?,
            current: decode_equipment_item_option(decoder)?,
        };
        validate_player_equipment_changed(&event)?;
        Ok(event)
    }
}

fn encode_equipment_item_option(
    item: &Option<PlayerEquipmentItemSnapshot>,
    encoder: &mut Encoder,
) -> SjmlResult<()> {
    let Some(item) = item else {
        return encoder.write_u8(0);
    };
    encoder.write_u8(1)?;
    encode_bounded_event_string(&item.item_id, encoder)?;
    encoder.write_u32(item.count)?;
    encoder.write_u32(item.damage)?;
    encoder.write_u32(item.max_damage)
}

fn decode_equipment_item_option(
    decoder: &mut Decoder<'_>,
) -> SjmlResult<Option<PlayerEquipmentItemSnapshot>> {
    match decoder.read_u8()? {
        0 => Ok(None),
        1 => Ok(Some(PlayerEquipmentItemSnapshot {
            item_id: decode_bounded_event_string(decoder)?,
            count: decoder.read_u32()?,
            damage: decoder.read_u32()?,
            max_damage: decoder.read_u32()?,
        })),
        value => Err(SjmlError::new(
            ErrorCode::InvalidBoolean,
            format!("optional item wire value must be 0 or 1, found {value}"),
        )),
    }
}

fn validate_player_equipment_changed(event: &PlayerEquipmentChanged) -> SjmlResult<()> {
    let invalid = || {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "player equipment event contains an invalid UUID, slot, item ID, count, or durability",
        )
    };
    if !canonical_uuid(&event.player_id) {
        return Err(invalid());
    }
    for item in [&event.previous, &event.current].into_iter().flatten() {
        if !canonical_resource_location(&item.item_id)
            || item.count == 0
            || item.count > MAX_PLAYER_EQUIPMENT_ITEM_COUNT
            || item.damage > item.max_damage
        {
            return Err(invalid());
        }
        if item.item_id.len() > 256 {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "player equipment item ID exceeds the 256-byte limit",
            ));
        }
    }
    Ok(())
}

fn canonical_uuid(value: &str) -> bool {
    value.len() == 36
        && value.bytes().enumerate().all(|(index, byte)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                byte == b'-'
            } else {
                byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)
            }
        })
}

fn canonical_resource_location(value: &str) -> bool {
    let Some((namespace, path)) = value.split_once(':') else {
        return false;
    };
    !namespace.is_empty()
        && !path.is_empty()
        && namespace.bytes().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'.' || b == b'-'
        })
        && path.bytes().all(|b| {
            b.is_ascii_lowercase()
                || b.is_ascii_digit()
                || b == b'_'
                || b == b'.'
                || b == b'-'
                || b == b'/'
        })
}

fn encode_bounded_event_string(value: &str, encoder: &mut Encoder) -> SjmlResult<()> {
    if value.len() > 256 {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "advancement progress event string exceeds the 256-byte limit",
        ));
    }
    encoder.write_length(value.len())?;
    encoder.write_raw(value.as_bytes())
}

fn decode_bounded_event_string(decoder: &mut Decoder<'_>) -> SjmlResult<String> {
    let length = decoder.read_length(256)?;
    String::from_utf8(decoder.read_raw(length)?.to_vec()).map_err(|_| {
        SjmlError::new(
            ErrorCode::InvalidUtf8,
            "advancement progress event string is not valid UTF-8",
        )
    })
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

/// Synchronous cancellable physical-client attempt to send the original chat text to the server.
///
/// Requires `client.chat.send.observe` to read the text, `events.subscribe` to receive the event,
/// and `events.cancel` to veto it. SJML never rewrites this message; a veto prevents sending before
/// Minecraft constructs the signed chat packet.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct ClientChatSendAttempt {
    /// Original plain text typed by the local user, before other mods change the outgoing value.
    pub message: String,
}

/// One of the supported physical-client vanilla interaction keymappings.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ClientAction {
    /// The vanilla attack mapping (the default left-mouse action).
    Attack,
    /// The vanilla use-item mapping, delivered for the hand reported by NeoForge.
    UseItem,
    /// The vanilla pick-block mapping (the default middle-mouse action).
    PickBlock,
}

impl StableEncode for ClientAction {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        encoder.write_u32(match self {
            Self::Attack => 0,
            Self::UseItem => 1,
            Self::PickBlock => 2,
        })
    }
}

impl StableDecode for ClientAction {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        match decoder.read_u32()? {
            0 => Ok(Self::Attack),
            1 => Ok(Self::UseItem),
            2 => Ok(Self::PickBlock),
            value => Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                format!("client action code {value} is not supported"),
            )),
        }
    }
}

/// Hand reported for the action mapping; attack and pick-block normally use `Main`.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ClientActionHand {
    /// Main hand.
    Main,
    /// Offhand.
    Off,
}

impl StableEncode for ClientActionHand {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        encoder.write_u32(match self {
            Self::Main => 0,
            Self::Off => 1,
        })
    }
}

impl StableDecode for ClientActionHand {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        match decoder.read_u32()? {
            0 => Ok(Self::Main),
            1 => Ok(Self::Off),
            value => Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                format!("client action hand code {value} is not supported"),
            )),
        }
    }
}

/// Synchronous cancellable attempt to run a supported vanilla interaction action on the client.
///
/// This detached value contains only the action and hand, never a Minecraft key, player, world,
/// or item. Return `Ok(true)` from a `client.action_attempt` event callback to cancel the mapping.
/// It requires a client-side module with `events.subscribe`, `events.cancel`, and the separate
/// `client.input.action_cancel` grant; at most 64 callbacks run for one input action.
#[derive(Debug, Clone, Copy, Eq, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct ClientActionAttempt {
    /// Supported vanilla action mapping that was triggered.
    pub action: ClientAction,
    /// Hand reported by NeoForge for this action mapping.
    pub hand: ClientActionHand,
}

/// Synchronous, non-cancellable notification for a button click on a module-owned client screen.
///
/// The host emits `client.screen.action` directly only to the owner of the screen that received
/// the click. It contains the stable screen and button IDs, not Minecraft objects or command text.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct ClientScreenAction {
    /// ID of the module-owned screen that was clicked.
    pub screen_id: String,
    /// ID of the button clicked on that screen.
    pub button_id: String,
}

/// One bounded value returned by a module-owned client form screen.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct ClientScreenFormValue {
    /// Stable ID of the input field in the form definition.
    pub id: String,
    /// Literal text entered by the user, or the selected option ID for a list field.
    /// An empty value means that a list field has no selection.
    pub value: String,
}

/// Synchronous, non-cancellable submit notification for a module-owned client form screen.
///
/// The host emits `client.screen.form_submit` directly only to the owner of the form that received
/// the click. It includes the clicked button, current bounded text values, any selected option ID
/// from a schema-v3/v4 selection list, and schema-v4 checkbox states as `"true"` or `"false"`.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct ClientScreenFormSubmit {
    /// ID of the module-owned form screen that was submitted.
    pub screen_id: String,
    /// ID of the button clicked on that form.
    pub button_id: String,
    /// Values in definition order: text fields, optional selection-list value, then checkboxes.
    pub fields: Vec<ClientScreenFormValue>,
}

/// Synchronous, non-cancellable submit notification for a schema-v6 multi-select form screen.
///
/// The host emits `client.screen.multi_select_form_submit` directly only to the owner of the open
/// form. The list ID identifies the control and selected option IDs are reported in declaration
/// order. Selection and scrolling remain local until a button is submitted.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 6)]
pub struct ClientScreenMultiSelectFormSubmit {
    /// ID of the module-owned form screen that was submitted.
    pub screen_id: String,
    /// ID of the button clicked on that form.
    pub button_id: String,
    /// Current bounded text fields in declaration order.
    pub fields: Vec<ClientScreenFormValue>,
    /// Stable ID of the submitted multi-select list.
    pub list_id: String,
    /// Selected option IDs in the order declared by the list.
    pub selected_option_ids: Vec<String>,
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

/// Synchronous cancellable attempt to use the item held by a logical-server player.
///
/// Subscribe to `player.item_use_attempt` with `events::subscribe` and return `Ok(true)` to veto
/// the item's use. NeoForge emits this snapshot before `Item::use`, after block and entity
/// interactions have been ruled out and after any `player.interaction` veto. It contains only
/// detached identifiers and a bounded count; no live item stack crosses the ABI. The callback
/// runs on the server thread and requires both `events.subscribe` and `events.cancel`.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerItemUseAttempt {
    /// Canonical UUID string of the player who attempted to use an item.
    pub player_id: String,
    /// Resource location of the level where the item-use attempt occurred.
    pub dimension_id: String,
    /// `main_hand` or `off_hand`.
    pub hand: String,
    /// Registry ID of the item in that hand; an empty hand is `minecraft:air` with count zero.
    pub item_id: String,
    /// Stack count in `0..=1_000_000`.
    pub count: u32,
}

/// Synchronous cancellable attempt to start drinking a potion on the logical server.
///
/// NeoForge emits `LivingEntityUseItemEvent.Start` before the item becomes the entity's active
/// use item, and cancellation prevents that start. The host filters to `PotionItem`, so milk,
/// food, bows, and other item-use starts are not included. Only detached values cross the ABI;
/// the callback runs on the server thread and requires `events.subscribe` plus `events.cancel`.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerPotionUseAttempt {
    /// Canonical UUID string of the player who began using the potion.
    pub player_id: String,
    /// Resource location of the level where potion use started.
    pub dimension_id: String,
    /// Registry ID of the potion item.
    pub item_id: String,
    /// Potion stack count in `1..=64`.
    pub count: u32,
    /// Remaining item-use duration in ticks in `1..=72_000`.
    pub duration_ticks: u32,
    /// `main_hand` or `off_hand`.
    pub hand: String,
}

impl StableEncode for PlayerPotionUseAttempt {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        validate_player_potion_use_attempt(self)?;
        encoder.write_i32(1)?;
        encode_bounded_event_string(&self.player_id, encoder)?;
        encode_bounded_event_string(&self.dimension_id, encoder)?;
        encode_bounded_event_string(&self.item_id, encoder)?;
        encoder.write_u32(self.count)?;
        encoder.write_u32(self.duration_ticks)?;
        encode_bounded_event_string(&self.hand, encoder)
    }
}

impl StableDecode for PlayerPotionUseAttempt {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        let schema_version = decoder.read_i32()?;
        if schema_version != 1 {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                format!("PlayerPotionUseAttempt schema version {schema_version} is unsupported"),
            ));
        }
        let event = Self {
            player_id: decode_bounded_event_string(decoder)?,
            dimension_id: decode_bounded_event_string(decoder)?,
            item_id: decode_bounded_event_string(decoder)?,
            count: decoder.read_u32()?,
            duration_ticks: decoder.read_u32()?,
            hand: decode_bounded_event_string(decoder)?,
        };
        validate_player_potion_use_attempt(&event)?;
        Ok(event)
    }
}

fn validate_player_potion_use_attempt(event: &PlayerPotionUseAttempt) -> SjmlResult<()> {
    if !canonical_uuid(&event.player_id)
        || !canonical_resource_location(&event.dimension_id)
        || !canonical_resource_location(&event.item_id)
        || !(event.hand == "main_hand" || event.hand == "off_hand")
        || event.count == 0
        || event.count > MAX_PLAYER_POTION_USE_ITEM_COUNT
        || event.duration_ticks == 0
        || event.duration_ticks > MAX_PLAYER_POTION_USE_DURATION_TICKS
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player potion-use attempt contains invalid detached player, item, hand, count, or duration data",
        ));
    }
    Ok(())
}

/// Synchronous decision returned by `player.potion_use_attempt`.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PlayerPotionUseAttemptAction {
    /// Allow the potion-use start to continue.
    Pass,
    /// Cancel the potion-use start before the potion can be consumed.
    Cancel,
}

impl StableEncode for PlayerPotionUseAttemptAction {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        encoder.write_u8(match self {
            Self::Pass => 0,
            Self::Cancel => 1,
        })
    }
}

impl StableDecode for PlayerPotionUseAttemptAction {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        match decoder.read_u8()? {
            0 => Ok(Self::Pass),
            1 => Ok(Self::Cancel),
            value => Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                format!("player potion-use action code {value} is not supported"),
            )),
        }
    }
}

/// Synchronous cancellable attempt to use a totem before it prevents a player's death.
///
/// NeoForge's logical-server `LivingUseTotemEvent` fires before the totem effect is applied.
/// Return [`PlayerTotemUseAttemptAction::Cancel`] to prevent the totem from saving the player.
/// The event includes the detached damage type and, when present, the causing entity identity,
/// but never exposes a Minecraft object, damage source, or mutable item stack. Both
/// `events.subscribe` and `events.cancel` are required.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerTotemUseAttempt {
    /// Canonical UUID string of the player whose totem would be used.
    pub player_id: String,
    /// Resource location of the player's current dimension.
    pub dimension_id: String,
    /// Registry ID of the damage type that would trigger the totem.
    pub damage_type_id: String,
    /// UUID of the causing entity, or `None` for environmental damage.
    pub source_entity_id: Option<String>,
    /// Registry ID of the causing entity type, or `None` for environmental damage.
    pub source_entity_type_id: Option<String>,
    /// Registry ID of the totem item being used.
    pub totem_id: String,
    /// Totem stack count, bounded by the host to `1..=64`.
    pub count: u32,
    /// `main_hand` or `off_hand`.
    pub hand: String,
}

impl StableEncode for PlayerTotemUseAttempt {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        validate_player_totem_use_attempt(self)?;
        encoder.write_i32(1)?;
        encode_bounded_event_string(&self.player_id, encoder)?;
        encode_bounded_event_string(&self.dimension_id, encoder)?;
        encode_bounded_event_string(&self.damage_type_id, encoder)?;
        encode_optional_bounded_event_string(&self.source_entity_id, encoder)?;
        encode_optional_bounded_event_string(&self.source_entity_type_id, encoder)?;
        encode_bounded_event_string(&self.totem_id, encoder)?;
        encoder.write_u32(self.count)?;
        encode_bounded_event_string(&self.hand, encoder)
    }
}

impl StableDecode for PlayerTotemUseAttempt {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        let schema_version = decoder.read_i32()?;
        if schema_version != 1 {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                format!("PlayerTotemUseAttempt schema version {schema_version} is unsupported"),
            ));
        }
        let event = Self {
            player_id: decode_bounded_event_string(decoder)?,
            dimension_id: decode_bounded_event_string(decoder)?,
            damage_type_id: decode_bounded_event_string(decoder)?,
            source_entity_id: decode_optional_bounded_event_string(decoder)?,
            source_entity_type_id: decode_optional_bounded_event_string(decoder)?,
            totem_id: decode_bounded_event_string(decoder)?,
            count: decoder.read_u32()?,
            hand: decode_bounded_event_string(decoder)?,
        };
        validate_player_totem_use_attempt(&event)?;
        Ok(event)
    }
}

fn encode_optional_bounded_event_string(
    value: &Option<String>,
    encoder: &mut Encoder,
) -> SjmlResult<()> {
    match value {
        Some(value) => encode_bounded_event_string(value, encoder),
        None => encoder.write_u32(u32::MAX),
    }
}

fn decode_optional_bounded_event_string(decoder: &mut Decoder<'_>) -> SjmlResult<Option<String>> {
    let length = decoder.read_u32()?;
    if length == u32::MAX {
        return Ok(None);
    }
    let length = usize::try_from(length).map_err(|_| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "optional totem event string length does not fit this platform",
        )
    })?;
    if length > 256 {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "optional totem event string exceeds the 256-byte limit",
        ));
    }
    String::from_utf8(decoder.read_raw(length)?.to_vec())
        .map(Some)
        .map_err(|_| {
            SjmlError::new(
                ErrorCode::InvalidUtf8,
                "optional totem event string is not valid UTF-8",
            )
        })
}

fn validate_player_totem_use_attempt(event: &PlayerTotemUseAttempt) -> SjmlResult<()> {
    let source_identity_is_valid = match (&event.source_entity_id, &event.source_entity_type_id) {
        (None, None) => true,
        (Some(entity_id), Some(entity_type_id)) => {
            canonical_uuid(entity_id) && canonical_resource_location(entity_type_id)
        }
        _ => false,
    };
    if !canonical_uuid(&event.player_id)
        || !canonical_resource_location(&event.dimension_id)
        || !canonical_resource_location(&event.damage_type_id)
        || !canonical_resource_location(&event.totem_id)
        || !source_identity_is_valid
        || !(event.hand == "main_hand" || event.hand == "off_hand")
        || event.count == 0
        || event.count > MAX_PLAYER_TOTEM_USE_COUNT
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player totem-use attempt contains invalid detached identity, item, hand, or count data",
        ));
    }
    Ok(())
}

/// Synchronous decision returned by `player.totem_use_attempt`.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PlayerTotemUseAttemptAction {
    /// Allow the totem effect to save the player.
    Pass,
    /// Cancel the totem effect; the pending damage/death proceeds normally.
    Cancel,
}

impl StableEncode for PlayerTotemUseAttemptAction {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        encoder.write_u8(match self {
            Self::Pass => 0,
            Self::Cancel => 1,
        })
    }
}

impl StableDecode for PlayerTotemUseAttemptAction {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        match decoder.read_u8()? {
            0 => Ok(Self::Pass),
            1 => Ok(Self::Cancel),
            value => Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                format!("player totem-use action code {value} is not supported"),
            )),
        }
    }
}

/// Synchronous use callback for an item declared by the receiving module.
///
/// The host routes `item.use` only to the module that declared the item ID in the package's
/// `package:module/path` namespace. The callback runs on the logical server before vanilla
/// `Item::use`, after `player.item_use_attempt`; it receives detached identifiers and may return
/// [`ItemUseAction`] to pass the use through, handle it, or consume exactly one held item. Guest
/// side effects can use the normal server APIs, with their usual permissions and validation.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct ModuleItemUse {
    /// Canonical UUID string of the player who attempted the use.
    pub player_id: String,
    /// Resource location of the level where the use occurred.
    pub dimension_id: String,
    /// `main_hand` or `off_hand`.
    pub hand: String,
    /// Canonical registry ID of the module-owned item.
    pub item_id: String,
    /// Item count in `0..=1_000_000` at the start of the callback.
    pub count: u32,
}

/// Synchronous use callback for a module-owned item right-clicked on a block.
///
/// The host routes `item.use_on_block` only to the module that declared the held item under its
/// `package:module/path` namespace. It runs on the logical server after `block.use` has passed and
/// before vanilla `Item::useOn`; the snapshot contains detached player, item, block, position,
/// face, and hand values. Return [`ItemUseAction`] to pass through, handle the interaction, or
/// consume one item if the held item ID and count remain unchanged.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct ModuleItemUseOnBlock {
    /// Canonical UUID string of the player who attempted the use.
    pub player_id: String,
    /// Resource location of the level where the interaction occurred.
    pub dimension_id: String,
    /// Canonical registry ID of the module-owned held item.
    pub item_id: String,
    /// Stack count in `0..=1_000_000` at the start of the callback.
    pub item_count: u32,
    /// Canonical registry ID of the clicked block.
    pub block_id: String,
    /// Clicked block X coordinate.
    pub x: i32,
    /// Clicked block Y coordinate.
    pub y: i32,
    /// Clicked block Z coordinate.
    pub z: i32,
    /// Vanilla face name: `down`, `up`, `north`, `south`, `west`, or `east`.
    pub face: String,
    /// `main_hand` or `off_hand`.
    pub hand: String,
}

/// Synchronous use callback for a module-owned item used on an entity.
///
/// The host routes `item.use_on_entity` only to the module that declared the held item under its
/// `package:module/path` namespace. It runs on the logical server after the general
/// `player.interaction` veto and after the more specific entity `interactAt` stage has passed,
/// before vanilla entity/item interaction behavior. The snapshot contains detached player, item,
/// target entity, position, and hand values. Return [`ItemUseAction`] to pass through, handle the
/// interaction, or consume one item if the held item ID and count remain unchanged.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct ModuleItemUseOnEntity {
    /// Canonical UUID string of the player who attempted the use.
    pub player_id: String,
    /// Resource location of the level where the interaction occurred.
    pub dimension_id: String,
    /// Canonical registry ID of the module-owned held item.
    pub item_id: String,
    /// Stack count in `0..=1_000_000` at the start of the callback.
    pub item_count: u32,
    /// Canonical UUID string of the targeted entity.
    pub target_entity_id: String,
    /// Canonical entity-type registry ID of the target.
    pub target_type_id: String,
    /// Target entity X coordinate at interaction time.
    pub target_x: f64,
    /// Target entity Y coordinate at interaction time.
    pub target_y: f64,
    /// Target entity Z coordinate at interaction time.
    pub target_z: f64,
    /// `main_hand` or `off_hand`.
    pub hand: String,
}

/// Synchronous, cancellable attack attempt made with an item owned by the receiving module.
///
/// The host routes `item.attack_attempt` only to the module that declared the held item in its
/// `package:module/path` namespace. It runs on the logical server after the general
/// `player.attack_attempt` callbacks have allowed the attack and before vanilla attack processing.
/// The snapshot contains detached player, dimension, item, and target values. Return `Ok(true)`
/// to veto the attack. Both `events.subscribe` and `events.cancel` are required, and the callback
/// must be synchronous because Minecraft is waiting to resolve the cancellable attack hook.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct ModuleItemAttackAttempt {
    /// Canonical UUID string of the player who initiated the attack.
    pub player_id: String,
    /// Resource location of the level containing the player and target.
    pub dimension_id: String,
    /// Canonical ID of the same-module registered item in the player's main hand.
    pub item_id: String,
    /// Main-hand item count at attack time, in `1..=99`.
    pub item_count: u32,
    /// Canonical UUID string of the targeted entity.
    pub target_id: String,
    /// Canonical registry ID of the targeted entity type.
    pub target_type_id: String,
    /// Target X coordinate when the attack hook ran.
    pub target_x: f64,
    /// Target Y coordinate when the attack hook ran.
    pub target_y: f64,
    /// Target Z coordinate when the attack hook ran.
    pub target_z: f64,
}

/// Synchronous use callback for a block declared by the receiving module.
///
/// The host routes `block.use` only to the module that declared the target block in the package's
/// `package:module/path` namespace. It runs on the logical server before vanilla block use and
/// after the generic `player.interaction` veto. The payload contains detached player, block
/// position, face, hand, and held-item values. Return [`BlockUseAction`] to pass the click through,
/// handle it, or consume exactly one unchanged held item.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct ModuleBlockUse {
    /// Canonical UUID string of the player who attempted the use.
    pub player_id: String,
    /// Resource location of the level where the use occurred.
    pub dimension_id: String,
    /// Canonical registry ID of the module-owned block.
    pub block_id: String,
    /// Block X coordinate.
    pub x: i32,
    /// Block Y coordinate.
    pub y: i32,
    /// Block Z coordinate.
    pub z: i32,
    /// Vanilla face name: `down`, `up`, `north`, `south`, `west`, or `east`.
    pub face: String,
    /// `main_hand` or `off_hand`.
    pub hand: String,
    /// Canonical registry ID of the item held at the start of the callback.
    pub item_id: String,
    /// Held stack count in `0..=1_000_000`.
    pub item_count: u32,
}

/// Server-authoritative outcome of a module-owned item use callback.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ItemUseAction {
    /// Continue with Minecraft's normal item use behavior.
    Pass,
    /// Handle the click and return success without changing the held stack.
    Success,
    /// Handle the click and consume one item if the original stack is unchanged.
    ConsumeOne,
}

/// Server-authoritative outcome of a module-owned block use callback.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockUseAction {
    /// Continue with Minecraft's normal block-use behavior.
    Pass,
    /// Handle the click without changing the held stack.
    Success,
    /// Handle the click and consume one item if the original stack is unchanged.
    ConsumeOne,
}

impl StableEncode for BlockUseAction {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        encoder.write_u8(match self {
            Self::Pass => 0,
            Self::Success => 1,
            Self::ConsumeOne => 2,
        })
    }
}

impl StableDecode for BlockUseAction {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        match decoder.read_u8()? {
            0 => Ok(Self::Pass),
            1 => Ok(Self::Success),
            2 => Ok(Self::ConsumeOne),
            value => Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                format!("block.use action code {value} is not supported"),
            )),
        }
    }
}

impl StableEncode for ItemUseAction {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        encoder.write_u8(match self {
            Self::Pass => 0,
            Self::Success => 1,
            Self::ConsumeOne => 2,
        })
    }
}

impl StableDecode for ItemUseAction {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        match decoder.read_u8()? {
            0 => Ok(Self::Pass),
            1 => Ok(Self::Success),
            2 => Ok(Self::ConsumeOne),
            value => Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                format!("item.use action code {value} is not supported"),
            )),
        }
    }
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

/// Detached client-side snapshot emitted while Minecraft builds a tooltip for an item stack.
///
/// Subscribe to `client.item.tooltip` with `events::subscribe`. The callback may return bounded
/// literal lines through [`ItemTooltipLines`]; changing tooltips requires the
/// `client.item.tooltip.modify` capability in addition to `events.subscribe`.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct ClientItemTooltip {
    /// Canonical registry ID of the item in the stack.
    pub item_id: String,
    /// Current stack count, in `1..=1_000_000`.
    pub count: u32,
    /// Current vanilla damage value, or zero for non-damageable stacks.
    pub damage: u32,
    /// The item's vanilla maximum damage value, or zero when it is not damageable.
    pub max_damage: u32,
}

/// Bounded literal lines to append after the vanilla item tooltip.
#[derive(Debug, Default, PartialEq, StableEncode)]
#[sjml(version = 1)]
pub struct ItemTooltipLines {
    lines: Vec<String>,
}

impl ItemTooltipLines {
    /// Maximum lines that one callback can add to a single tooltip.
    pub const MAX_LINES: usize = 16;
    /// Maximum UTF-8 bytes in one added line.
    pub const MAX_LINE_BYTES: usize = 256;
    /// Maximum combined UTF-8 bytes in all lines returned by one callback.
    pub const MAX_TOTAL_BYTES: usize = 2_048;

    pub fn new() -> Self {
        Self::default()
    }

    /// Adds one literal tooltip line. Control characters and empty lines are rejected.
    pub fn push(&mut self, line: impl Into<String>) -> SjmlResult<()> {
        let line = line.into();
        if line.is_empty()
            || line.len() > Self::MAX_LINE_BYTES
            || line.chars().any(char::is_control)
        {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "tooltip lines must contain 1–256 UTF-8 bytes without control characters",
            ));
        }
        let total_bytes = self.lines.iter().map(String::len).sum::<usize>() + line.len();
        if self.lines.len() >= Self::MAX_LINES || total_bytes > Self::MAX_TOTAL_BYTES {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "tooltip callback exceeds its line or byte limit",
            ));
        }
        self.lines.push(line);
        Ok(())
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }
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

/// Synchronous cancellable attempt to pick up a detached item stack on the logical server.
///
/// Return `Ok(true)` from a `player.item_pickup_attempt` handler to prevent the item from being
/// inserted into the player's inventory. The callback runs before vanilla pickup rules are applied,
/// receives only the player UUID, canonical item ID/count, and position, and requires both
/// `events.subscribe` and `events.cancel`. It cannot modify the item stack or player inventory.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerItemPickupAttempt {
    /// Canonical UUID string of the player who touched the item.
    pub player_id: String,
    /// Canonical registry ID of the item being picked up.
    pub item_id: String,
    /// Item count, bounded by the host to `1..=1_000_000`.
    pub count: u32,
    /// Item-entity X coordinate at the start of the pickup attempt.
    pub x: f64,
    /// Item-entity Y coordinate at the start of the pickup attempt.
    pub y: f64,
    /// Item-entity Z coordinate at the start of the pickup attempt.
    pub z: f64,
}

/// One detached item ID and count from a fishing catch, without components or NBT.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct FishedDropSnapshot {
    /// Canonical registry ID of the caught item.
    pub item_id: String,
    /// Item count, bounded to `1..=1_000_000`.
    pub count: u32,
}

/// Synchronous cancellable logical-server attempt to collect items from a fishing catch.
///
/// Return `Ok(true)` from a `player.fished_attempt` handler to cancel the catch. NeoForge then
/// gives the player no drops but still applies the event's rod damage. This callback cannot
/// change drops or rod damage. It receives at most 16 detached item ID/count pairs, no arbitrary
/// item NBT, and requires both `events.subscribe` and `events.cancel`.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerFishedAttempt {
    /// Canonical UUID string of the player who made the catch.
    pub player_id: String,
    /// Resource location of the dimension containing the fishing hook.
    pub dimension_id: String,
    /// Fishing-hook X coordinate at the event.
    pub hook_x: f64,
    /// Fishing-hook Y coordinate at the event.
    pub hook_y: f64,
    /// Fishing-hook Z coordinate at the event.
    pub hook_z: f64,
    /// Bounded rod damage requested by NeoForge; the callback cannot change it.
    pub rod_damage: u32,
    /// Up to 16 detached item registry IDs and counts, without item components or NBT.
    pub drops: Vec<FishedDropSnapshot>,
    /// Whether the event's drop list had more entries than this bounded snapshot includes.
    pub drops_truncated: bool,
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

/// Synchronous cancellable fall attempt for a loaded living entity on the logical server.
///
/// NeoForge 21.1.265 fires this before applying fall damage. Return `Ok(true)` from an
/// `entity.fall_attempt` handler to cancel the fall, which sets NeoForge's fall distance to zero.
/// The snapshot contains detached entity identity, position, fall distance, and the current damage
/// multiplier; it cannot change the multiplier or any other fall fields. Both `events.subscribe`
/// and `events.cancel` are required.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityFallAttempt {
    /// Canonical UUID of the living entity that is about to take fall damage.
    pub entity_id: String,
    /// Registry ID of the living entity type.
    pub entity_type_id: String,
    /// Resource location of the dimension containing the entity.
    pub dimension_id: String,
    /// Entity X coordinate at the time the attempt was dispatched.
    pub x: f64,
    /// Entity Y coordinate at the time the attempt was dispatched.
    pub y: f64,
    /// Entity Z coordinate at the time the attempt was dispatched.
    pub z: f64,
    /// Fall distance passed to NeoForge's damage hook.
    pub fall_distance: f32,
    /// Current multiplier applied to fall distance to calculate damage.
    pub damage_multiplier: f32,
}

/// Synchronous cancellable attempt to mount or dismount an entity on the logical server.
///
/// NeoForge fires `entity.mount_attempt` before changing either entity's riding relationship.
/// Return `Ok(true)` to cancel the attempted mount or dismount. The detached snapshot carries the
/// acting entity, an optional mounted entity, the dimension, and the action direction. Both
/// `events.subscribe` and `events.cancel` are required, and the callback must remain synchronous.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityMountAttempt {
    /// Canonical UUID of the entity attempting to mount or dismount.
    pub entity_id: String,
    /// Registry ID of the acting entity type.
    pub entity_type_id: String,
    /// Resource location of the entities' level.
    pub dimension_id: String,
    /// Entity being mounted; NeoForge may omit it for a dismount event.
    pub mounted_entity: Option<EntityMountTarget>,
    /// True for a mount attempt and false for a dismount attempt.
    pub mounting: bool,
}

/// Detached identity of the entity that a mount attempt targets.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityMountTarget {
    /// Canonical UUID of the target entity.
    pub entity_id: String,
    /// Registry ID of the target entity type.
    pub entity_type_id: String,
}

/// Synchronous cancellable knockback attempt for a living entity on the logical server.
///
/// NeoForge 21.1.265 fires this before applying knockback. The event exposes original and current
/// strength/direction values for observation only: this SJML callback may veto the operation but
/// cannot mutate those values. Return `Ok(true)` from `entity.knockback_attempt` to cancel it.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityKnockbackAttempt {
    /// Canonical UUID string of the living entity that would be pushed.
    pub entity_id: String,
    /// Registry ID of the entity type.
    pub entity_type_id: String,
    /// Resource location of the entity's level.
    pub dimension_id: String,
    /// Entity X coordinate at the time the attempt was dispatched.
    pub x: f64,
    /// Entity Y coordinate at the time the attempt was dispatched.
    pub y: f64,
    /// Entity Z coordinate at the time the attempt was dispatched.
    pub z: f64,
    /// Knockback strength before earlier event listeners changed it.
    pub original_strength: f32,
    /// Current requested knockback strength.
    pub strength: f32,
    /// Original X direction ratio.
    pub original_ratio_x: f64,
    /// Original Z direction ratio.
    pub original_ratio_z: f64,
    /// Current X direction ratio.
    pub ratio_x: f64,
    /// Current Z direction ratio.
    pub ratio_z: f64,
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

/// Synchronous, cancellable attempt to spawn an offspring on a logical server.
///
/// Subscribe to `entity.breed_attempt` and return `Ok(true)` to suppress the proposed baby. The
/// event uses only detached parent and child UUIDs, entity type IDs, and the dimension ID; it never
/// exposes a Minecraft object. It runs once on the server owner thread and requires both
/// `events.subscribe` and `events.cancel`.
#[derive(Clone, Debug, Eq, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityBreedAttempt {
    /// Canonical UUID string of the initiating parent.
    pub parent_a_id: String,
    /// Registry ID of the initiating parent's entity type.
    pub parent_a_type_id: String,
    /// Canonical UUID string of the secondary parent.
    pub parent_b_id: String,
    /// Registry ID of the secondary parent's entity type.
    pub parent_b_type_id: String,
    /// Canonical UUID string of the proposed offspring.
    pub child_id: String,
    /// Registry ID of the proposed offspring's entity type.
    pub child_type_id: String,
    /// Resource location of the level where both parents and the child are located.
    pub dimension_id: String,
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

/// Synchronous, non-cancellable snapshot of a logical-server player tick.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerTick {
    /// Canonical UUID string of the player whose tick callback was delivered.
    pub player_id: String,
    /// Resource location of the level where the player is currently located.
    pub dimension_id: String,
    /// Logical server tick count when this callback was delivered.
    pub tick_count: i64,
}

/// Synchronous, non-cancellable snapshot of a physical-client tick.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct ClientTick {
    /// Physical client tick count when this callback was delivered.
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

/// Synchronous, non-cancellable snapshot emitted after the player takes a crafting result.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerItemCrafted {
    /// Canonical UUID string of the player who crafted the result.
    pub player_id: String,
    /// Registry ID of the recipe whose inputs matched the crafting matrix.
    pub recipe_id: String,
    /// Registry ID of the output item.
    pub item_id: String,
    /// Number of output items taken from the crafting result slot.
    pub count: i32,
}

/// Synchronous, non-cancellable snapshot emitted after a player completes a trade with a villager
/// or wandering trader. The NeoForge event runs after the merchant accepted the inputs and updated
/// the offer use count. IDs and stack counts are detached; item components and mutable Minecraft
/// objects are not exposed.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerVillagerTraded {
    /// Canonical UUID string of the player who completed the trade.
    pub player_id: String,
    /// Canonical UUID string of the merchant entity.
    pub merchant_id: String,
    /// Registry ID of the merchant entity type, commonly `minecraft:villager` or `minecraft:wandering_trader`.
    pub merchant_type_id: String,
    /// Dimension ID where the trade occurred.
    pub dimension_id: String,
    /// Merchant block X coordinate at the trade.
    pub x: i32,
    /// Merchant block Y coordinate at the trade.
    pub y: i32,
    /// Merchant block Z coordinate at the trade.
    pub z: i32,
    /// Villager profession registry ID, or empty for wandering traders and merchants without a profession.
    pub profession_id: String,
    /// Villager level in `1..=5`, or zero when no profession is available.
    pub profession_level: u32,
    /// Registry ID of the first item cost after the current offer price modifiers.
    pub cost_a_item_id: String,
    /// Count taken for the first item cost.
    pub cost_a_count: u32,
    /// Registry ID of the second cost item, or empty when the trade has one cost.
    pub cost_b_item_id: String,
    /// Count taken for the second cost, or zero when `cost_b_item_id` is empty.
    pub cost_b_count: u32,
    /// Registry ID of the traded result item.
    pub result_item_id: String,
    /// Count in the traded result stack.
    pub result_count: u32,
    /// Offer use count after this successful trade.
    pub uses: u32,
    /// Maximum use count declared by the offer.
    pub max_uses: u32,
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

/// Detached registry ID, count and durability values for a block-drop tool or result.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct BlockDropsItemSnapshot {
    /// Canonical registry ID of the item, without components or NBT.
    pub item_id: String,
    /// Stack count, bounded by the host to `1..=1_000_000`.
    pub count: u32,
    /// Current item damage, present only when the item uses durability.
    pub damage: Option<u32>,
    /// Maximum durability, present together with `damage` for damageable items.
    pub max_damage: Option<u32>,
}

/// Detached identity of the entity that broke a block, when NeoForge provides one.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct BlockDropsBreakerSnapshot {
    /// Canonical UUID string of the breaker.
    pub entity_id: String,
    /// Canonical registry ID of the breaker's entity type.
    pub entity_type_id: String,
}

/// Synchronous cancellable snapshot of already-computed block drops on the logical server.
///
/// Return `Ok(true)` from a `#[event(name = "block.drops_attempt")]` handler to cancel the
/// item and experience drops. NeoForge fires this after computing drops and before spawning
/// them; the host leaves the NeoForge event and block's post-break hook active, so the block break
/// itself is not rolled back. The payload contains only bounded IDs,
/// counts, durability, position and experience, never live Minecraft objects, item components or
/// NBT. A veto requires both `events.subscribe` and `events.cancel`.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct BlockDropsAttempt {
    /// Canonical resource location of the dimension containing the broken block.
    pub dimension_id: String,
    /// Position of the block whose computed drops are being considered.
    pub position: BlockPos,
    /// Registry ID of the block type; block-state properties are omitted.
    pub block_id: String,
    /// Identity of the breaker, if the event supplied one.
    pub breaker: Option<BlockDropsBreakerSnapshot>,
    /// Detached tool ID/count/durability, absent if no tool was used.
    pub tool: Option<BlockDropsItemSnapshot>,
    /// Up to 32 detached computed drops; item components and NBT are omitted.
    pub drops: Vec<BlockDropsItemSnapshot>,
    /// Computed experience amount, bounded by the host to `0..=1_000_000`.
    pub experience: u32,
    /// Whether the original drop list contained more entries than this bounded snapshot.
    pub drops_truncated: bool,
}

/// Synchronous, non-cancellable callback for one scheduled tick of a module-owned registered block.
///
/// The host dispatches only to the module that declared `block_id`, requires `events.subscribe`,
/// and caps total `block.tick` callbacks at 512 for each logical-server tick. This event is emitted
/// after Minecraft invokes the registered block's scheduled `tick` method; it is not a random-tick
/// callback. Property values are detached strings sorted by name. The callback runs on the
/// server thread and may use explicitly granted world APIs to read or change the loaded world.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct BlockTick {
    /// Canonical ID of the registered block whose scheduled tick ran.
    pub block_id: String,
    /// Resource location of the dimension containing the block.
    pub dimension_id: String,
    /// Block position where Minecraft invoked the scheduled tick.
    pub position: BlockPos,
    /// Logical-server tick count at dispatch time.
    pub tick_count: i64,
    /// Detached property values from the exact block state being ticked.
    pub properties: Vec<BlockTickProperty>,
}

impl BlockTick {
    /// Returns the value of a captured property, if the block declared it.
    pub fn property(&self, name: &str) -> Option<&str> {
        self.properties
            .iter()
            .find(|property| property.name == name)
            .map(|property| property.value.as_str())
    }
}

/// Synchronous, non-cancellable callback for one vanilla random tick of a module-owned registered block.
///
/// Opt a block in during `#[mod_register]` with
/// [`registry::set_block_random_ticking`], then subscribe to the distinct
/// `block.random_tick` event. The host dispatches only to the module that declared
/// `block_id`, on the logical server thread, and caps the combined `block.tick`
/// and `block.random_tick` callbacks at 512 for each server tick. Snapshot fields
/// use the same bounded schema-v1 layout as [`BlockTick`], but this event has its
/// own event name and ABI export. No client-side callback is emitted.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct BlockRandomTick {
    /// Canonical ID of the registered block whose random tick ran.
    pub block_id: String,
    /// Resource location of the dimension containing the block.
    pub dimension_id: String,
    /// Block position where Minecraft invoked the random tick.
    pub position: BlockPos,
    /// Logical-server tick count at dispatch time.
    pub tick_count: i64,
    /// Detached property values from the exact block state being ticked.
    pub properties: Vec<BlockTickProperty>,
}

impl BlockRandomTick {
    /// Returns the value of a captured property, if the block declared it.
    pub fn property(&self, name: &str) -> Option<&str> {
        self.properties
            .iter()
            .find(|property| property.name == name)
            .map(|property| property.value.as_str())
    }
}

/// One named, detached property on a block state delivered in [`BlockTick`] or [`BlockRandomTick`].
#[derive(Debug, Clone, Eq, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct BlockTickProperty {
    /// Canonical property name.
    pub name: String,
    /// Canonical value, such as `true` or `3`.
    pub value: String,
}

/// Synchronous cancellable attempt to grow a block on the logical server.
///
/// Return `Ok(true)` from a `block.grow_attempt` handler to set the NeoForge pre-event result to
/// `DO_NOT_GROW`. Returning false leaves the existing result unchanged, so vanilla's `DEFAULT`
/// behavior remains in effect and this callback never forces growth. The snapshot contains only
/// a dimension ID, block position, and block registry ID; it does not expose the block state or a
/// live Minecraft object. Both `events.subscribe` and `events.cancel` are required.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct BlockGrowAttempt {
    /// Resource location of the dimension where the block attempted to grow.
    pub dimension_id: String,
    /// Position of the block that is attempting to advance its growth state.
    pub position: BlockPos,
    /// Registry ID of the current block; state properties such as age are omitted.
    pub block_id: String,
}

/// Synchronous cancellable attempt to apply bone meal on the logical server.
///
/// Return `Ok(true)` from a `block.fertilize_attempt` handler to cancel the original
/// NeoForge `BonemealEvent` and report the attempt as unsuccessful. Vanilla then
/// leaves the fertilizer stack untouched. Returning false preserves the existing
/// event result. The payload contains detached IDs/counts and a block position;
/// it does not expose the player, item stack, block state, or level object. Both
/// `events.subscribe` and `events.cancel` are required.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct BlockFertilizeAttempt {
    /// UUID of the player who used fertilizer, or `None` for non-player sources.
    pub player_id: Option<String>,
    /// Resource location of the dimension where the fertilizer was used.
    pub dimension_id: String,
    /// Position of the block that received the fertilizer attempt.
    pub position: BlockPos,
    /// Registry ID of the current block; block-state properties are omitted.
    pub block_id: String,
    /// Registry ID of the fertilizer item used by the attempt.
    pub fertilizer_item_id: String,
    /// Number of items in the detached fertilizer stack, bounded to 0 through 99.
    pub fertilizer_count: u32,
    /// Whether Minecraft considered this block a valid bone-meal target.
    pub valid_target: bool,
}

/// Synchronous cancellable attempt to trample farmland on the logical server.
///
/// Return `Ok(true)` from a `block.farmland_trample_attempt` handler to cancel
/// NeoForge's `FarmlandTrampleEvent`, preserving the farmland block. Returning
/// false leaves vanilla's result unchanged. The detached snapshot carries the
/// dimension, position, block ID, entity UUID and type ID, and fall distance;
/// it exposes no live Minecraft objects or block-state properties. Both
/// `events.subscribe` and `events.cancel` are required.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct BlockFarmlandTrampleAttempt {
    /// Resource location of the dimension where the entity landed.
    pub dimension_id: String,
    /// Position of the farmland block that may be trampled.
    pub position: BlockPos,
    /// Registry ID of the block at `position` when the attempt was captured.
    pub block_id: String,
    /// Canonical UUID of the entity that landed on the farmland.
    pub entity_id: String,
    /// Registry ID of the entity type that attempted to trample the farmland.
    pub entity_type_id: String,
    /// Fall distance reported by NeoForge for this attempt.
    pub fall_distance: f32,
}

/// Synchronous cancellable attempt by a fluid to place or replace a block on the logical server.
///
/// Return `Ok(true)` from a `block.fluid_place_attempt` handler to cancel this one placement.
/// A veto prevents the block state change that the fluid was about to make; it does not stop the
/// fluid itself from flowing or alter later ticks. The snapshot carries only detached registry
/// IDs and positions, with block-state properties omitted. Both `events.subscribe` and
/// `events.cancel` are required.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct BlockFluidPlaceAttempt {
    /// Resource location of the dimension where the block would be placed.
    pub dimension_id: String,
    /// Position whose block state the fluid attempted to replace.
    pub target_position: BlockPos,
    /// Position of the fluid state that caused the placement; may equal `target_position`.
    pub liquid_position: BlockPos,
    /// Registry ID of the fluid type that caused the placement.
    pub fluid_id: String,
    /// Registry ID of the block state present before the attempt.
    pub original_block_id: String,
    /// Registry ID of the block state Minecraft intended to place.
    pub result_block_id: String,
}

/// Synchronous cancellable attempt by a player to attack an entity on the logical server.
///
/// Return `Ok(true)` from a `player.attack_attempt` handler to cancel this attack before
/// Minecraft runs weapon hooks or damage processing. The payload contains detached attacker and
/// target UUIDs, target type and dimension IDs, and the target's current position. It does not
/// expose damage, a live player/entity object, or mutable event fields. Both `events.subscribe`
/// and `events.cancel` are required.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerAttackAttempt {
    /// UUID of the player who initiated the attack.
    pub player_id: String,
    /// UUID of the targeted entity.
    pub target_id: String,
    /// Registry ID of the targeted entity type.
    pub target_type_id: String,
    /// Resource location of the dimension containing both entities.
    pub dimension_id: String,
    /// Target's X coordinate when the attack hook ran.
    pub target_x: f64,
    /// Target's Y coordinate when the attack hook ran.
    pub target_y: f64,
    /// Target's Z coordinate when the attack hook ran.
    pub target_z: f64,
}

/// Synchronous cancellable snapshot of a logical-server XP change request.
///
/// The callback receives only detached identity and XP values. Return a
/// [`PlayerExperienceChangeAction`] to continue, cancel, or replace the requested delta.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerExperienceChangeAttempt {
    /// Canonical UUID string of the player whose XP is changing.
    pub player_id: String,
    /// Player level before this XP change is applied.
    pub level_before: i32,
    /// Progress toward the next level before this XP change is applied, in `0.0..=1.0`.
    pub progress_before: f32,
    /// XP delta requested by Minecraft before callbacks run.
    pub requested_amount: i32,
}

impl StableEncode for PlayerExperienceChangeAttempt {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        validate_player_experience_change_attempt(self)?;
        encoder.write_i32(1)?;
        encode_bounded_event_string(&self.player_id, encoder)?;
        encoder.write_i32(self.level_before)?;
        encoder.write_f32(self.progress_before)?;
        encoder.write_i32(self.requested_amount)
    }
}

impl StableDecode for PlayerExperienceChangeAttempt {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        let schema_version = decoder.read_i32()?;
        if schema_version != 1 {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                format!(
                    "PlayerExperienceChangeAttempt schema version {schema_version} is unsupported"
                ),
            ));
        }
        let event = Self {
            player_id: decode_bounded_event_string(decoder)?,
            level_before: decoder.read_i32()?,
            progress_before: decoder.read_f32()?,
            requested_amount: decoder.read_i32()?,
        };
        validate_player_experience_change_attempt(&event)?;
        Ok(event)
    }
}

fn validate_player_experience_change_attempt(
    event: &PlayerExperienceChangeAttempt,
) -> SjmlResult<()> {
    if !canonical_uuid(&event.player_id)
        || event.level_before < 0
        || event.level_before > MAX_PLAYER_EXPERIENCE_LEVEL
        || !event.progress_before.is_finite()
        || !(0.0..=1.0).contains(&event.progress_before)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player experience change attempt contains invalid detached player XP data",
        ));
    }
    if event.player_id.len() > 256 {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "player experience change attempt UUID exceeds the 256-byte limit",
        ));
    }
    Ok(())
}

/// Synchronous server-thread decision returned by `player.experience_change_attempt`.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PlayerExperienceChangeAction {
    /// Keep the originally requested XP delta and allow later handlers to decide.
    Pass,
    /// Cancel this XP change.
    Cancel,
    /// Replace the requested XP delta with a bounded amount.
    SetAmount(i32),
}

impl StableEncode for PlayerExperienceChangeAction {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        match self {
            Self::Pass => encoder.write_u8(0),
            Self::Cancel => encoder.write_u8(1),
            Self::SetAmount(amount) => {
                validate_player_experience_change_amount(*amount)?;
                encoder.write_u8(2)?;
                encoder.write_i32(*amount)
            }
        }
    }
}

impl StableDecode for PlayerExperienceChangeAction {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        match decoder.read_u8()? {
            0 => Ok(Self::Pass),
            1 => Ok(Self::Cancel),
            2 => {
                let amount = decoder.read_i32()?;
                validate_player_experience_change_amount(amount)?;
                Ok(Self::SetAmount(amount))
            }
            value => Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                format!("player experience change action code {value} is not supported"),
            )),
        }
    }
}

fn validate_player_experience_change_amount(amount: i32) -> SjmlResult<()> {
    if !(MIN_PLAYER_EXPERIENCE_CHANGE_AMOUNT..=MAX_PLAYER_EXPERIENCE_CHANGE_AMOUNT)
        .contains(&amount)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            format!(
                "player experience change amount must be between {MIN_PLAYER_EXPERIENCE_CHANGE_AMOUNT} and {MAX_PLAYER_EXPERIENCE_CHANGE_AMOUNT}"
            ),
        ));
    }
    Ok(())
}

/// Synchronous cancellable snapshot of a logical-server XP level change request.
///
/// The callback receives only detached identity and level values. Return a
/// [`PlayerExperienceLevelChangeAction`] to continue, cancel, or replace the requested delta.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerExperienceLevelChangeAttempt {
    /// Canonical UUID string of the player whose level is changing.
    pub player_id: String,
    /// Player level before this level delta is applied.
    pub level_before: i32,
    /// Level delta requested by Minecraft before callbacks run.
    pub requested_levels: i32,
}

impl StableEncode for PlayerExperienceLevelChangeAttempt {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        validate_player_experience_level_change_attempt(self)?;
        encoder.write_i32(1)?;
        encode_bounded_event_string(&self.player_id, encoder)?;
        encoder.write_i32(self.level_before)?;
        encoder.write_i32(self.requested_levels)
    }
}

impl StableDecode for PlayerExperienceLevelChangeAttempt {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        let schema_version = decoder.read_i32()?;
        if schema_version != 1 {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                format!(
                    "PlayerExperienceLevelChangeAttempt schema version {schema_version} is unsupported"
                ),
            ));
        }
        let event = Self {
            player_id: decode_bounded_event_string(decoder)?,
            level_before: decoder.read_i32()?,
            requested_levels: decoder.read_i32()?,
        };
        validate_player_experience_level_change_attempt(&event)?;
        Ok(event)
    }
}

fn validate_player_experience_level_change_attempt(
    event: &PlayerExperienceLevelChangeAttempt,
) -> SjmlResult<()> {
    if !canonical_uuid(&event.player_id)
        || event.level_before < 0
        || event.level_before > MAX_PLAYER_EXPERIENCE_LEVEL_SNAPSHOT
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player experience level change attempt contains invalid detached player data",
        ));
    }
    if event.player_id.len() > 256 {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "player experience level change attempt UUID exceeds the 256-byte limit",
        ));
    }
    Ok(())
}

/// Synchronous server-thread decision returned by `player.experience_level_change_attempt`.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PlayerExperienceLevelChangeAction {
    /// Keep the originally requested level delta and allow later handlers to decide.
    Pass,
    /// Cancel this level change.
    Cancel,
    /// Replace the requested level delta with a bounded value.
    SetLevels(i32),
}

impl StableEncode for PlayerExperienceLevelChangeAction {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        match self {
            Self::Pass => encoder.write_u8(0),
            Self::Cancel => encoder.write_u8(1),
            Self::SetLevels(levels) => {
                validate_player_experience_level_delta(*levels)?;
                encoder.write_u8(2)?;
                encoder.write_i32(*levels)
            }
        }
    }
}

impl StableDecode for PlayerExperienceLevelChangeAction {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        match decoder.read_u8()? {
            0 => Ok(Self::Pass),
            1 => Ok(Self::Cancel),
            2 => {
                let levels = decoder.read_i32()?;
                validate_player_experience_level_delta(levels)?;
                Ok(Self::SetLevels(levels))
            }
            value => Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                format!("player experience level change action code {value} is not supported"),
            )),
        }
    }
}

fn validate_player_experience_level_delta(levels: i32) -> SjmlResult<()> {
    if !(MIN_PLAYER_EXPERIENCE_LEVEL_DELTA..=MAX_PLAYER_EXPERIENCE_LEVEL_DELTA).contains(&levels) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            format!(
                "player experience level delta must be between {MIN_PLAYER_EXPERIENCE_LEVEL_DELTA} and {MAX_PLAYER_EXPERIENCE_LEVEL_DELTA}"
            ),
        ));
    }
    Ok(())
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

/// Synchronous, non-cancellable snapshot of one loaded non-player entity tick.
#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityTick {
    /// Canonical lowercase UUID of the entity.
    pub entity_id: String,
    /// Registry ID of the entity type.
    pub entity_type_id: String,
    /// Registry ID of the dimension containing the entity.
    pub dimension_id: String,
    /// Entity X coordinate after this entity tick.
    pub x: f64,
    /// Entity Y coordinate after this entity tick.
    pub y: f64,
    /// Entity Z coordinate after this entity tick.
    pub z: f64,
    /// Number of entity ticks since this entity instance was created or loaded.
    pub tick_count: i64,
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
/// callback and returns its scoped handle. Plain server, player, entity, and client tick
/// subscriptions run every 20 ticks; use [`subscribe_with_interval`] to choose a cadence.
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

/// Registers a server, player, entity, or client tick callback at the requested cadence
/// (1–1,200 ticks).
pub fn subscribe_with_interval(
    event_name: &str,
    priority: EventPriority,
    interval_ticks: u32,
) -> SjmlResult<EventSubscriptionHandle> {
    validate_event_name(event_name)?;
    if !matches!(
        event_name,
        "server.tick" | "player.tick" | "entity.tick" | "client.tick"
    ) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "interval subscriptions are only supported for server.tick, player.tick, entity.tick and client.tick",
        ));
    }
    if !(1..=MAX_SERVER_TICK_INTERVAL).contains(&interval_ticks) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "tick event interval must be between 1 and 1200 ticks",
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
        subscribe, subscribe_with_interval, unsubscribe, AdvancementProgressOperation, BlockBreak,
        BlockBreakAttempt, BlockDropsAttempt, BlockDropsBreakerSnapshot, BlockDropsItemSnapshot,
        BlockFarmlandTrampleAttempt, BlockFertilizeAttempt, BlockFluidPlaceAttempt,
        BlockGrowAttempt, BlockPlace, BlockPlaceAttempt, BlockRandomTick, BlockTick,
        BlockTickProperty, BlockUseAction, ClientAction, ClientActionAttempt, ClientActionHand,
        ClientChatSendAttempt, ClientItemTooltip, ClientTick, EffectTickEvent, EntityBreedAttempt,
        EntityDamageAttempt, EntityDamaged, EntityDeath, EntityDeathAttempt,
        EntityEffectApplicationAttempt, EntityFallAttempt, EntityHealAttempt, EntityJoinAttempt,
        EntityKnockbackAttempt, EntityMountAttempt, EntityMountTarget, EntitySpawned,
        EntityTargetAttempt, EntityTick, EventPriority, ExplosionAttempt, FishedDropSnapshot,
        InventoryChanged, ItemTooltipLines, ItemUseAction, ModuleBlockUse, ModuleItemAttackAttempt,
        ModuleItemUse, ModuleItemUseOnBlock, ModuleItemUseOnEntity, PlayerAdvancementEarned,
        PlayerAdvancementProgressEvent, PlayerAttackAttempt, PlayerDropAttempt,
        PlayerEquipmentChanged, PlayerEquipmentItemSnapshot, PlayerEquipmentSlot,
        PlayerExperienceChangeAction, PlayerExperienceChangeAttempt,
        PlayerExperienceLevelChangeAction, PlayerExperienceLevelChangeAttempt, PlayerFishedAttempt,
        PlayerInteracted, PlayerInteractionAttempt, PlayerItemCrafted, PlayerItemPickupAttempt,
        PlayerItemUseAttempt, PlayerMoved, PlayerPotionUseAttempt, PlayerPotionUseAttemptAction,
        PlayerRespawned, PlayerSleepAttempt, PlayerTick, PlayerTotemUseAttempt,
        PlayerTotemUseAttemptAction, PlayerVillagerTraded, ProjectileImpactAttempt,
        ResourcesReloaded, ServerTick, WorldLifecycle, MAX_PLAYER_EQUIPMENT_ITEM_COUNT,
        MAX_PLAYER_EXPERIENCE_CHANGE_AMOUNT, MAX_PLAYER_EXPERIENCE_LEVEL_DELTA,
        MAX_PLAYER_EXPERIENCE_LEVEL_SNAPSHOT, MAX_PLAYER_POTION_USE_DURATION_TICKS,
        MAX_PLAYER_POTION_USE_ITEM_COUNT, MAX_PLAYER_TOTEM_USE_COUNT, MAX_SERVER_TICK_INTERVAL,
        MIN_PLAYER_EXPERIENCE_CHANGE_AMOUNT, MIN_PLAYER_EXPERIENCE_LEVEL_DELTA,
    };
    use crate::inventory::BlockPos;
    use crate::ErrorCode;
    use crate::{from_bytes, to_bytes};

    #[test]
    fn client_item_tooltip_round_trips_and_bounds_appended_literal_lines() {
        let event = ClientItemTooltip {
            item_id: "minecraft:diamond_sword".to_owned(),
            count: 1,
            damage: 7,
            max_damage: 1_561,
        };
        let encoded = to_bytes(&event).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_u32.to_le_bytes());
        expected.extend_from_slice(&(event.item_id.len() as u32).to_le_bytes());
        expected.extend_from_slice(event.item_id.as_bytes());
        expected.extend_from_slice(&event.count.to_le_bytes());
        expected.extend_from_slice(&event.damage.to_le_bytes());
        expected.extend_from_slice(&event.max_damage.to_le_bytes());
        assert_eq!(encoded, expected);
        assert_eq!(from_bytes::<ClientItemTooltip>(&encoded).unwrap(), event);

        let mut lines = ItemTooltipLines::new();
        lines.push("Quality: epic").unwrap();
        lines.push("Bound to this character").unwrap();
        let encoded = to_bytes(&lines).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_u32.to_le_bytes());
        expected.extend_from_slice(&2_u32.to_le_bytes());
        for line in lines.lines() {
            expected.extend_from_slice(&(line.len() as u32).to_le_bytes());
            expected.extend_from_slice(line.as_bytes());
        }
        assert_eq!(encoded, expected);
        assert!(lines.lines().contains(&"Quality: epic".to_owned()));
    }

    #[test]
    fn client_item_tooltip_lines_reject_controls_and_excessive_output() {
        let mut lines = ItemTooltipLines::new();
        assert_eq!(
            lines.push("line\nwith control").unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            lines
                .push("x".repeat(ItemTooltipLines::MAX_LINE_BYTES + 1))
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        for index in 0..ItemTooltipLines::MAX_LINES {
            lines.push(format!("line {index}")).unwrap();
        }
        assert_eq!(
            lines.push("extra").unwrap_err().code(),
            ErrorCode::PayloadTooLarge
        );

        let mut total_limited = ItemTooltipLines::new();
        for _ in 0..8 {
            total_limited.push("x".repeat(256)).unwrap();
        }
        assert_eq!(
            total_limited.push("x").unwrap_err().code(),
            ErrorCode::PayloadTooLarge
        );
    }

    #[test]
    fn outgoing_chat_attempt_round_trips_original_text() {
        let attempt = ClientChatSendAttempt {
            message: "Привет 👋".into(),
        };
        let mut expected = 1_u32.to_le_bytes().to_vec();
        let text = attempt.message.as_bytes();
        expected.extend_from_slice(&(text.len() as u32).to_le_bytes());
        expected.extend_from_slice(text);
        let encoded = to_bytes(&attempt).unwrap();
        assert_eq!(encoded, expected);
        assert_eq!(
            from_bytes::<ClientChatSendAttempt>(&encoded).unwrap(),
            attempt
        );
    }

    #[test]
    fn client_action_attempt_matches_host_wire_codes_and_round_trips() {
        let attempt = ClientActionAttempt {
            action: ClientAction::UseItem,
            hand: ClientActionHand::Off,
        };
        let mut expected = 1_u32.to_le_bytes().to_vec();
        expected.extend_from_slice(&1_u32.to_le_bytes());
        expected.extend_from_slice(&1_u32.to_le_bytes());
        let encoded = to_bytes(&attempt).unwrap();
        assert_eq!(encoded, expected);
        assert_eq!(
            from_bytes::<ClientActionAttempt>(&encoded).unwrap(),
            attempt
        );

        let mut invalid_action = 1_u32.to_le_bytes().to_vec();
        invalid_action.extend_from_slice(&9_u32.to_le_bytes());
        invalid_action.extend_from_slice(&0_u32.to_le_bytes());
        assert_eq!(
            from_bytes::<ClientActionAttempt>(&invalid_action)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }

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
    fn advancement_progress_round_trips_operation_and_post_change_completion() {
        let event = PlayerAdvancementProgressEvent {
            player_id: "00000000-0000-0000-0000-000000000001".into(),
            advancement_id: "minecraft:story/root".into(),
            criterion_id: "crafting_table".into(),
            operation: AdvancementProgressOperation::Grant,
            is_done: true,
        };
        let mut expected = 1_u32.to_le_bytes().to_vec();
        for value in [&event.player_id, &event.advancement_id, &event.criterion_id] {
            expected.extend_from_slice(&(value.len() as u32).to_le_bytes());
            expected.extend_from_slice(value.as_bytes());
        }
        expected.extend_from_slice(&0_u32.to_le_bytes());
        expected.push(1);
        assert_eq!(to_bytes(&event).unwrap(), expected);
        assert_eq!(
            from_bytes::<PlayerAdvancementProgressEvent>(&expected).unwrap(),
            event
        );

        let revoked = PlayerAdvancementProgressEvent {
            operation: AdvancementProgressOperation::Revoke,
            is_done: false,
            ..event
        };
        assert_eq!(
            from_bytes::<PlayerAdvancementProgressEvent>(&to_bytes(&revoked).unwrap()).unwrap(),
            revoked
        );
    }

    #[test]
    fn advancement_progress_codec_rejects_bad_identity_operation_boolean_utf8_and_bounds() {
        let base = PlayerAdvancementProgressEvent {
            player_id: "00000000-0000-0000-0000-000000000001".into(),
            advancement_id: "minecraft:story/root".into(),
            criterion_id: "criterion".into(),
            operation: AdvancementProgressOperation::Grant,
            is_done: false,
        };
        let invalid_id = PlayerAdvancementProgressEvent {
            player_id: "not-a-uuid".into(),
            ..base.clone()
        };
        assert_eq!(
            to_bytes(&invalid_id).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        let invalid_resource = PlayerAdvancementProgressEvent {
            advancement_id: "Minecraft:root".into(),
            ..base.clone()
        };
        assert_eq!(
            to_bytes(&invalid_resource).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        let oversized = PlayerAdvancementProgressEvent {
            criterion_id: "x".repeat(257),
            ..base
        };
        assert_eq!(
            to_bytes(&oversized).unwrap_err().code(),
            ErrorCode::PayloadTooLarge
        );

        let mut invalid_operation = to_bytes(&PlayerAdvancementProgressEvent {
            player_id: "00000000-0000-0000-0000-000000000001".into(),
            advancement_id: "minecraft:story/root".into(),
            criterion_id: "criterion".into(),
            operation: AdvancementProgressOperation::Grant,
            is_done: false,
        })
        .unwrap();
        let operation_offset = invalid_operation.len() - 5;
        invalid_operation[operation_offset..operation_offset + 4]
            .copy_from_slice(&99_u32.to_le_bytes());
        assert_eq!(
            from_bytes::<PlayerAdvancementProgressEvent>(&invalid_operation)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );

        let mut invalid_boolean = to_bytes(&PlayerAdvancementProgressEvent {
            player_id: "00000000-0000-0000-0000-000000000001".into(),
            advancement_id: "minecraft:story/root".into(),
            criterion_id: "criterion".into(),
            operation: AdvancementProgressOperation::Grant,
            is_done: false,
        })
        .unwrap();
        let boolean_offset = invalid_boolean.len() - 1;
        invalid_boolean[boolean_offset] = 2;
        assert_eq!(
            from_bytes::<PlayerAdvancementProgressEvent>(&invalid_boolean)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidBoolean
        );

        let mut invalid_utf8 = to_bytes(&PlayerAdvancementProgressEvent {
            player_id: "00000000-0000-0000-0000-000000000001".into(),
            advancement_id: "minecraft:story/root".into(),
            criterion_id: "criterion".into(),
            operation: AdvancementProgressOperation::Grant,
            is_done: false,
        })
        .unwrap();
        let criterion_offset = 4 + 4 + 36 + 4 + "minecraft:story/root".len() + 4;
        invalid_utf8[criterion_offset] = 0xff;
        assert_eq!(
            from_bytes::<PlayerAdvancementProgressEvent>(&invalid_utf8)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidUtf8
        );
    }

    #[test]
    fn player_equipment_change_round_trips_stable_slots_and_bounded_old_new_stacks() {
        let event = PlayerEquipmentChanged {
            player_id: "00000000-0000-0000-0000-000000000001".into(),
            slot: PlayerEquipmentSlot::MainHand,
            previous: Some(PlayerEquipmentItemSnapshot {
                item_id: "minecraft:iron_sword".into(),
                count: 1,
                damage: 5,
                max_damage: 250,
            }),
            current: Some(PlayerEquipmentItemSnapshot {
                item_id: "minecraft:diamond_sword".into(),
                count: 1,
                damage: 2,
                max_damage: 1_561,
            }),
        };

        let mut expected = 1_u32.to_le_bytes().to_vec();
        expected.extend_from_slice(&(event.player_id.len() as u32).to_le_bytes());
        expected.extend_from_slice(event.player_id.as_bytes());
        expected.extend_from_slice(&0_u32.to_le_bytes());
        for item in [
            event.previous.as_ref().unwrap(),
            event.current.as_ref().unwrap(),
        ] {
            expected.push(1);
            expected.extend_from_slice(&(item.item_id.len() as u32).to_le_bytes());
            expected.extend_from_slice(item.item_id.as_bytes());
            expected.extend_from_slice(&item.count.to_le_bytes());
            expected.extend_from_slice(&item.damage.to_le_bytes());
            expected.extend_from_slice(&item.max_damage.to_le_bytes());
        }
        assert_eq!(to_bytes(&event).unwrap(), expected);
        assert_eq!(
            from_bytes::<PlayerEquipmentChanged>(&expected).unwrap(),
            event
        );

        let empty_to_equipped = PlayerEquipmentChanged {
            slot: PlayerEquipmentSlot::Head,
            previous: None,
            ..event
        };
        assert_eq!(
            from_bytes::<PlayerEquipmentChanged>(&to_bytes(&empty_to_equipped).unwrap()).unwrap(),
            empty_to_equipped
        );
    }

    #[test]
    fn player_equipment_change_rejects_invalid_identity_slot_option_item_and_bounds() {
        let base = PlayerEquipmentChanged {
            player_id: "00000000-0000-0000-0000-000000000001".into(),
            slot: PlayerEquipmentSlot::OffHand,
            previous: None,
            current: Some(PlayerEquipmentItemSnapshot {
                item_id: "minecraft:shield".into(),
                count: 1,
                damage: 0,
                max_damage: 0,
            }),
        };
        let invalid_uuid = PlayerEquipmentChanged {
            player_id: "not-a-uuid".into(),
            ..base.clone()
        };
        assert_eq!(
            to_bytes(&invalid_uuid).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        let invalid_id = PlayerEquipmentChanged {
            current: Some(PlayerEquipmentItemSnapshot {
                item_id: "Minecraft:shield".into(),
                ..base.current.clone().unwrap()
            }),
            ..base.clone()
        };
        assert_eq!(
            to_bytes(&invalid_id).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        let zero_count = PlayerEquipmentChanged {
            current: Some(PlayerEquipmentItemSnapshot {
                count: 0,
                ..base.current.clone().unwrap()
            }),
            ..base.clone()
        };
        assert_eq!(
            to_bytes(&zero_count).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        let oversized_count = PlayerEquipmentChanged {
            current: Some(PlayerEquipmentItemSnapshot {
                count: MAX_PLAYER_EQUIPMENT_ITEM_COUNT + 1,
                ..base.current.clone().unwrap()
            }),
            ..base.clone()
        };
        assert_eq!(
            to_bytes(&oversized_count).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        let invalid_damage = PlayerEquipmentChanged {
            current: Some(PlayerEquipmentItemSnapshot {
                damage: 1,
                ..base.current.clone().unwrap()
            }),
            ..base
        };
        assert_eq!(
            to_bytes(&invalid_damage).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );

        let mut invalid_slot = to_bytes(&PlayerEquipmentChanged {
            player_id: "00000000-0000-0000-0000-000000000001".into(),
            slot: PlayerEquipmentSlot::MainHand,
            previous: None,
            current: None,
        })
        .unwrap();
        invalid_slot[44..48].copy_from_slice(&99_u32.to_le_bytes());
        assert_eq!(
            from_bytes::<PlayerEquipmentChanged>(&invalid_slot)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );

        let mut invalid_option = to_bytes(&PlayerEquipmentChanged {
            player_id: "00000000-0000-0000-0000-000000000001".into(),
            slot: PlayerEquipmentSlot::MainHand,
            previous: None,
            current: None,
        })
        .unwrap();
        invalid_option[48] = 2;
        assert_eq!(
            from_bytes::<PlayerEquipmentChanged>(&invalid_option)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidBoolean
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
    fn block_drops_attempt_matches_host_schema_and_keeps_only_bounded_item_snapshots() {
        let attempt = BlockDropsAttempt {
            dimension_id: "minecraft:overworld".to_owned(),
            position: BlockPos { x: -3, y: 64, z: 8 },
            block_id: "minecraft:stone".to_owned(),
            breaker: Some(BlockDropsBreakerSnapshot {
                entity_id: "00000000-0000-0000-0000-000000000001".to_owned(),
                entity_type_id: "minecraft:player".to_owned(),
            }),
            tool: Some(BlockDropsItemSnapshot {
                item_id: "minecraft:diamond_pickaxe".to_owned(),
                count: 1,
                damage: Some(4),
                max_damage: Some(1_561),
            }),
            drops: vec![
                BlockDropsItemSnapshot {
                    item_id: "minecraft:cobblestone".to_owned(),
                    count: 2,
                    damage: None,
                    max_damage: None,
                },
                BlockDropsItemSnapshot {
                    item_id: "minecraft:diamond".to_owned(),
                    count: 1,
                    damage: None,
                    max_damage: None,
                },
            ],
            experience: 5,
            drops_truncated: false,
        };

        let encoded = to_bytes(&attempt).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &attempt.dimension_id);
        expected.extend_from_slice(&1_i32.to_le_bytes()); // nested BlockPos schema
        expected.extend_from_slice(&attempt.position.x.to_le_bytes());
        expected.extend_from_slice(&attempt.position.y.to_le_bytes());
        expected.extend_from_slice(&attempt.position.z.to_le_bytes());
        append_test_string(&mut expected, &attempt.block_id);
        expected.push(1);
        expected.extend_from_slice(&1_i32.to_le_bytes()); // nested breaker schema
        append_test_string(&mut expected, "00000000-0000-0000-0000-000000000001");
        append_test_string(&mut expected, "minecraft:player");
        expected.push(1);
        append_test_item_snapshot(&mut expected, attempt.tool.as_ref().unwrap());
        expected.extend_from_slice(&2_u32.to_le_bytes());
        for drop in &attempt.drops {
            append_test_item_snapshot(&mut expected, drop);
        }
        expected.extend_from_slice(&attempt.experience.to_le_bytes());
        expected.push(0);

        assert_eq!(
            encoded, expected,
            "Rust encoding must match host schema-v1 bytes"
        );
        assert_eq!(from_bytes::<BlockDropsAttempt>(&encoded).unwrap(), attempt);
    }

    fn append_test_item_snapshot(output: &mut Vec<u8>, item: &BlockDropsItemSnapshot) {
        output.extend_from_slice(&1_i32.to_le_bytes()); // nested item snapshot schema
        append_test_string(output, &item.item_id);
        output.extend_from_slice(&item.count.to_le_bytes());
        append_test_optional_u32(output, item.damage);
        append_test_optional_u32(output, item.max_damage);
    }

    fn append_test_optional_u32(output: &mut Vec<u8>, value: Option<u32>) {
        match value {
            Some(value) => {
                output.push(1);
                output.extend_from_slice(&value.to_le_bytes());
            }
            None => output.push(0),
        }
    }

    #[test]
    fn block_tick_matches_host_schema_and_exposes_state_properties() {
        let event = BlockTick {
            block_id: "ruby_pack:content/machine".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            position: BlockPos { x: -3, y: 70, z: 9 },
            tick_count: 42,
            properties: vec![
                BlockTickProperty {
                    name: "charge".to_owned(),
                    value: "3".to_owned(),
                },
                BlockTickProperty {
                    name: "powered".to_owned(),
                    value: "true".to_owned(),
                },
            ],
        };
        let encoded = to_bytes(&event).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &event.block_id);
        append_test_string(&mut expected, &event.dimension_id);
        expected.extend_from_slice(&1_i32.to_le_bytes());
        expected.extend_from_slice(&event.position.x.to_le_bytes());
        expected.extend_from_slice(&event.position.y.to_le_bytes());
        expected.extend_from_slice(&event.position.z.to_le_bytes());
        expected.extend_from_slice(&event.tick_count.to_le_bytes());
        expected.extend_from_slice(&2_i32.to_le_bytes());
        expected.extend_from_slice(&1_i32.to_le_bytes());
        append_test_string(&mut expected, "charge");
        append_test_string(&mut expected, "3");
        expected.extend_from_slice(&1_i32.to_le_bytes());
        append_test_string(&mut expected, "powered");
        append_test_string(&mut expected, "true");
        assert_eq!(
            encoded, expected,
            "Rust encoding must match the host v1 schema"
        );
        assert_eq!(from_bytes::<BlockTick>(&encoded).unwrap(), event);
        assert_eq!(event.property("powered"), Some("true"));
        assert_eq!(event.property("missing"), None);
    }

    #[test]
    fn block_random_tick_has_distinct_subscription_name_and_matches_snapshot_schema() {
        let event = BlockRandomTick {
            block_id: "ruby_pack:content/random_block".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            position: BlockPos { x: -3, y: 70, z: 9 },
            tick_count: 42,
            properties: vec![
                BlockTickProperty {
                    name: "charge".to_owned(),
                    value: "3".to_owned(),
                },
                BlockTickProperty {
                    name: "powered".to_owned(),
                    value: "true".to_owned(),
                },
            ],
        };
        let encoded = to_bytes(&event).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &event.block_id);
        append_test_string(&mut expected, &event.dimension_id);
        expected.extend_from_slice(&1_i32.to_le_bytes());
        expected.extend_from_slice(&event.position.x.to_le_bytes());
        expected.extend_from_slice(&event.position.y.to_le_bytes());
        expected.extend_from_slice(&event.position.z.to_le_bytes());
        expected.extend_from_slice(&event.tick_count.to_le_bytes());
        expected.extend_from_slice(&2_i32.to_le_bytes());
        expected.extend_from_slice(&1_i32.to_le_bytes());
        append_test_string(&mut expected, "charge");
        append_test_string(&mut expected, "3");
        expected.extend_from_slice(&1_i32.to_le_bytes());
        append_test_string(&mut expected, "powered");
        append_test_string(&mut expected, "true");
        assert_eq!(encoded, expected);
        assert_eq!(from_bytes::<BlockRandomTick>(&encoded).unwrap(), event);
        assert_eq!(event.property("powered"), Some("true"));
        assert_eq!(event.property("missing"), None);
    }

    #[test]
    fn block_grow_attempt_matches_host_schema_v1_bytes() {
        let event = BlockGrowAttempt {
            dimension_id: "minecraft:overworld".to_owned(),
            position: BlockPos {
                x: -4,
                y: 72,
                z: 19,
            },
            block_id: "minecraft:wheat".to_owned(),
        };
        let encoded = to_bytes(&event).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &event.dimension_id);
        expected.extend_from_slice(&1_i32.to_le_bytes());
        expected.extend_from_slice(&event.position.x.to_le_bytes());
        expected.extend_from_slice(&event.position.y.to_le_bytes());
        expected.extend_from_slice(&event.position.z.to_le_bytes());
        append_test_string(&mut expected, &event.block_id);
        assert_eq!(
            encoded, expected,
            "Rust encoding must match host schema-v1 bytes"
        );
        assert_eq!(from_bytes::<BlockGrowAttempt>(&encoded).unwrap(), event);
    }

    #[test]
    fn block_fertilize_attempt_matches_host_schema_v1_bytes() {
        let event = BlockFertilizeAttempt {
            player_id: Some("00000000-0000-0000-0000-000000000001".to_owned()),
            dimension_id: "minecraft:overworld".to_owned(),
            position: BlockPos {
                x: -4,
                y: 72,
                z: 19,
            },
            block_id: "minecraft:wheat".to_owned(),
            fertilizer_item_id: "minecraft:bone_meal".to_owned(),
            fertilizer_count: 3,
            valid_target: true,
        };
        let encoded = to_bytes(&event).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        expected.push(1);
        append_test_string(&mut expected, event.player_id.as_deref().unwrap());
        append_test_string(&mut expected, &event.dimension_id);
        expected.extend_from_slice(&1_i32.to_le_bytes());
        expected.extend_from_slice(&event.position.x.to_le_bytes());
        expected.extend_from_slice(&event.position.y.to_le_bytes());
        expected.extend_from_slice(&event.position.z.to_le_bytes());
        append_test_string(&mut expected, &event.block_id);
        append_test_string(&mut expected, &event.fertilizer_item_id);
        expected.extend_from_slice(&event.fertilizer_count.to_le_bytes());
        expected.push(1);
        assert_eq!(
            encoded, expected,
            "Rust encoding must match host schema-v1 bytes"
        );
        assert_eq!(
            from_bytes::<BlockFertilizeAttempt>(&encoded).unwrap(),
            event
        );

        let dispenser = BlockFertilizeAttempt {
            player_id: None,
            valid_target: false,
            ..event
        };
        let encoded = to_bytes(&dispenser).unwrap();
        assert_eq!(encoded[4], 0, "non-player attempts encode a missing actor");
        assert_eq!(
            from_bytes::<BlockFertilizeAttempt>(&encoded).unwrap(),
            dispenser
        );
    }

    #[test]
    fn block_farmland_trample_attempt_matches_host_schema_v1_bytes() {
        let event = BlockFarmlandTrampleAttempt {
            dimension_id: "minecraft:overworld".to_owned(),
            position: BlockPos {
                x: -4,
                y: 72,
                z: 19,
            },
            block_id: "minecraft:farmland".to_owned(),
            entity_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            entity_type_id: "minecraft:pig".to_owned(),
            fall_distance: 5.0,
        };
        let encoded = to_bytes(&event).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &event.dimension_id);
        expected.extend_from_slice(&1_i32.to_le_bytes());
        expected.extend_from_slice(&event.position.x.to_le_bytes());
        expected.extend_from_slice(&event.position.y.to_le_bytes());
        expected.extend_from_slice(&event.position.z.to_le_bytes());
        append_test_string(&mut expected, &event.block_id);
        append_test_string(&mut expected, &event.entity_id);
        append_test_string(&mut expected, &event.entity_type_id);
        expected.extend_from_slice(&event.fall_distance.to_le_bytes());
        assert_eq!(
            encoded, expected,
            "Rust encoding must match the host schema-v1 bytes"
        );
        assert_eq!(
            from_bytes::<BlockFarmlandTrampleAttempt>(&encoded).unwrap(),
            event
        );
    }

    #[test]
    fn block_fluid_place_attempt_matches_host_schema_v1_bytes() {
        let event = BlockFluidPlaceAttempt {
            dimension_id: "minecraft:overworld".to_owned(),
            target_position: BlockPos {
                x: -4,
                y: 72,
                z: 19,
            },
            liquid_position: BlockPos {
                x: -5,
                y: 72,
                z: 19,
            },
            fluid_id: "minecraft:lava".to_owned(),
            original_block_id: "minecraft:air".to_owned(),
            result_block_id: "minecraft:fire".to_owned(),
        };
        let encoded = to_bytes(&event).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &event.dimension_id);
        for position in [event.target_position, event.liquid_position] {
            expected.extend_from_slice(&1_i32.to_le_bytes());
            expected.extend_from_slice(&position.x.to_le_bytes());
            expected.extend_from_slice(&position.y.to_le_bytes());
            expected.extend_from_slice(&position.z.to_le_bytes());
        }
        append_test_string(&mut expected, &event.fluid_id);
        append_test_string(&mut expected, &event.original_block_id);
        append_test_string(&mut expected, &event.result_block_id);
        assert_eq!(
            encoded, expected,
            "Rust encoding must match host schema-v1 bytes"
        );
        assert_eq!(
            from_bytes::<BlockFluidPlaceAttempt>(&encoded).unwrap(),
            event
        );
    }

    #[test]
    fn player_attack_attempt_matches_host_schema_v1_bytes() {
        let event = PlayerAttackAttempt {
            player_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            target_id: "00000000-0000-0000-0000-000000000002".to_owned(),
            target_type_id: "minecraft:pig".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            target_x: -4.5,
            target_y: 72.0,
            target_z: 19.25,
        };
        let encoded = to_bytes(&event).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &event.player_id);
        append_test_string(&mut expected, &event.target_id);
        append_test_string(&mut expected, &event.target_type_id);
        append_test_string(&mut expected, &event.dimension_id);
        expected.extend_from_slice(&event.target_x.to_le_bytes());
        expected.extend_from_slice(&event.target_y.to_le_bytes());
        expected.extend_from_slice(&event.target_z.to_le_bytes());
        assert_eq!(
            encoded, expected,
            "Rust encoding must match host schema-v1 bytes"
        );
        assert_eq!(from_bytes::<PlayerAttackAttempt>(&encoded).unwrap(), event);
    }

    #[test]
    fn player_experience_change_attempt_matches_host_schema_v1_bytes() {
        let event = PlayerExperienceChangeAttempt {
            player_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            level_before: 27,
            progress_before: 0.375,
            requested_amount: 9,
        };
        let encoded = to_bytes(&event).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &event.player_id);
        expected.extend_from_slice(&event.level_before.to_le_bytes());
        expected.extend_from_slice(&event.progress_before.to_le_bytes());
        expected.extend_from_slice(&event.requested_amount.to_le_bytes());
        assert_eq!(
            encoded, expected,
            "Rust encoding must match host schema-v1 bytes"
        );
        assert_eq!(
            from_bytes::<PlayerExperienceChangeAttempt>(&encoded).unwrap(),
            event
        );

        let invalid = PlayerExperienceChangeAttempt {
            progress_before: f32::NAN,
            ..event
        };
        assert!(to_bytes(&invalid).is_err());
    }

    #[test]
    fn player_experience_change_action_uses_bounded_unversioned_tags() {
        for (action, expected) in [
            (PlayerExperienceChangeAction::Pass, vec![0]),
            (PlayerExperienceChangeAction::Cancel, vec![1]),
            (
                PlayerExperienceChangeAction::SetAmount(-17),
                [vec![2], (-17_i32).to_le_bytes().to_vec()].concat(),
            ),
        ] {
            let encoded = to_bytes(&action).unwrap();
            assert_eq!(encoded, expected);
            assert_eq!(
                from_bytes::<PlayerExperienceChangeAction>(&encoded).unwrap(),
                action
            );
        }

        assert!(to_bytes(&PlayerExperienceChangeAction::SetAmount(
            MAX_PLAYER_EXPERIENCE_CHANGE_AMOUNT + 1
        ))
        .is_err());
        assert!(to_bytes(&PlayerExperienceChangeAction::SetAmount(
            MIN_PLAYER_EXPERIENCE_CHANGE_AMOUNT - 1
        ))
        .is_err());

        for amount in [
            MAX_PLAYER_EXPERIENCE_CHANGE_AMOUNT + 1,
            MIN_PLAYER_EXPERIENCE_CHANGE_AMOUNT - 1,
        ] {
            let mut invalid = vec![2];
            invalid.extend_from_slice(&amount.to_le_bytes());
            assert!(from_bytes::<PlayerExperienceChangeAction>(&invalid).is_err());
        }
        assert!(from_bytes::<PlayerExperienceChangeAction>(&[3]).is_err());
    }

    #[test]
    fn player_experience_level_change_attempt_matches_host_schema_v1_bytes() {
        let event = PlayerExperienceLevelChangeAttempt {
            player_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            level_before: 27,
            requested_levels: 5,
        };
        let encoded = to_bytes(&event).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &event.player_id);
        expected.extend_from_slice(&event.level_before.to_le_bytes());
        expected.extend_from_slice(&event.requested_levels.to_le_bytes());
        assert_eq!(
            encoded, expected,
            "Rust encoding must match host schema-v1 bytes"
        );
        assert_eq!(
            from_bytes::<PlayerExperienceLevelChangeAttempt>(&encoded).unwrap(),
            event
        );

        let invalid = PlayerExperienceLevelChangeAttempt {
            level_before: MAX_PLAYER_EXPERIENCE_LEVEL_SNAPSHOT + 1,
            ..event
        };
        assert!(to_bytes(&invalid).is_err());
    }

    #[test]
    fn player_experience_level_change_action_uses_bounded_unversioned_tags() {
        for (action, expected) in [
            (PlayerExperienceLevelChangeAction::Pass, vec![0]),
            (PlayerExperienceLevelChangeAction::Cancel, vec![1]),
            (
                PlayerExperienceLevelChangeAction::SetLevels(-17),
                [vec![2], (-17_i32).to_le_bytes().to_vec()].concat(),
            ),
        ] {
            let encoded = to_bytes(&action).unwrap();
            assert_eq!(encoded, expected);
            assert_eq!(
                from_bytes::<PlayerExperienceLevelChangeAction>(&encoded).unwrap(),
                action
            );
        }

        for levels in [
            MIN_PLAYER_EXPERIENCE_LEVEL_DELTA - 1,
            MAX_PLAYER_EXPERIENCE_LEVEL_DELTA + 1,
        ] {
            assert!(to_bytes(&PlayerExperienceLevelChangeAction::SetLevels(levels)).is_err());
            let mut invalid = vec![2];
            invalid.extend_from_slice(&levels.to_le_bytes());
            assert!(from_bytes::<PlayerExperienceLevelChangeAction>(&invalid).is_err());
        }
        assert!(to_bytes(&PlayerExperienceLevelChangeAction::SetLevels(
            MIN_PLAYER_EXPERIENCE_LEVEL_DELTA
        ))
        .is_ok());
        assert!(to_bytes(&PlayerExperienceLevelChangeAction::SetLevels(
            MAX_PLAYER_EXPERIENCE_LEVEL_DELTA
        ))
        .is_ok());
        assert!(from_bytes::<PlayerExperienceLevelChangeAction>(&[3]).is_err());
    }

    #[test]
    fn module_item_attack_attempt_matches_host_schema_v1_bytes() {
        let event = ModuleItemAttackAttempt {
            player_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            item_id: "test_pack:combat/sabre".to_owned(),
            item_count: 2,
            target_id: "00000000-0000-0000-0000-000000000002".to_owned(),
            target_type_id: "minecraft:pig".to_owned(),
            target_x: -4.5,
            target_y: 72.0,
            target_z: 19.25,
        };
        let encoded = to_bytes(&event).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &event.player_id);
        append_test_string(&mut expected, &event.dimension_id);
        append_test_string(&mut expected, &event.item_id);
        expected.extend_from_slice(&event.item_count.to_le_bytes());
        append_test_string(&mut expected, &event.target_id);
        append_test_string(&mut expected, &event.target_type_id);
        expected.extend_from_slice(&event.target_x.to_le_bytes());
        expected.extend_from_slice(&event.target_y.to_le_bytes());
        expected.extend_from_slice(&event.target_z.to_le_bytes());
        assert_eq!(
            encoded, expected,
            "Rust bytes must match host schema-v1 layout"
        );
        assert_eq!(
            from_bytes::<ModuleItemAttackAttempt>(&encoded).unwrap(),
            event
        );
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
    fn cancellable_entity_fall_attempt_matches_host_schema_v1_bytes() {
        let event = EntityFallAttempt {
            entity_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            entity_type_id: "minecraft:pig".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            x: -1.5,
            y: 64.0,
            z: 2.25,
            fall_distance: 7.5,
            damage_multiplier: 1.0,
        };
        let encoded = to_bytes(&event).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &event.entity_id);
        append_test_string(&mut expected, &event.entity_type_id);
        append_test_string(&mut expected, &event.dimension_id);
        expected.extend_from_slice(&(-1.5_f64).to_le_bytes());
        expected.extend_from_slice(&(64.0_f64).to_le_bytes());
        expected.extend_from_slice(&(2.25_f64).to_le_bytes());
        expected.extend_from_slice(&(7.5_f32).to_le_bytes());
        expected.extend_from_slice(&(1.0_f32).to_le_bytes());
        assert_eq!(
            encoded, expected,
            "Rust bytes must match host schema-v1 layout"
        );
        assert_eq!(from_bytes::<EntityFallAttempt>(&encoded).unwrap(), event);
    }

    #[test]
    fn cancellable_entity_mount_attempt_encodes_optional_target_and_action() {
        let event = EntityMountAttempt {
            entity_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            entity_type_id: "minecraft:pig".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            mounted_entity: Some(EntityMountTarget {
                entity_id: "00000000-0000-0000-0000-000000000002".to_owned(),
                entity_type_id: "minecraft:minecart".to_owned(),
            }),
            mounting: true,
        };
        let encoded = to_bytes(&event).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &event.entity_id);
        append_test_string(&mut expected, &event.entity_type_id);
        append_test_string(&mut expected, &event.dimension_id);
        expected.push(1);
        expected.extend_from_slice(&1_i32.to_le_bytes());
        let mounted = event.mounted_entity.as_ref().unwrap();
        append_test_string(&mut expected, &mounted.entity_id);
        append_test_string(&mut expected, &mounted.entity_type_id);
        expected.push(1);
        assert_eq!(
            encoded, expected,
            "Rust bytes must match host schema-v1 layout"
        );
        assert_eq!(from_bytes::<EntityMountAttempt>(&encoded).unwrap(), event);

        let dismount = EntityMountAttempt {
            mounted_entity: None,
            mounting: false,
            ..event
        };
        let encoded = to_bytes(&dismount).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &dismount.entity_id);
        append_test_string(&mut expected, &dismount.entity_type_id);
        append_test_string(&mut expected, &dismount.dimension_id);
        expected.extend_from_slice(&[0, 0]);
        assert_eq!(encoded, expected);
        assert_eq!(
            from_bytes::<EntityMountAttempt>(&encoded).unwrap(),
            dismount
        );
    }

    #[test]
    fn cancellable_entity_knockback_attempt_matches_host_schema_v1_bytes() {
        let event = EntityKnockbackAttempt {
            entity_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            entity_type_id: "minecraft:pig".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            x: -1.5,
            y: 64.0,
            z: 2.25,
            original_strength: 0.8,
            strength: 0.5,
            original_ratio_x: 1.0,
            original_ratio_z: -2.0,
            ratio_x: 0.75,
            ratio_z: -1.5,
        };
        let encoded = to_bytes(&event).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &event.entity_id);
        append_test_string(&mut expected, &event.entity_type_id);
        append_test_string(&mut expected, &event.dimension_id);
        expected.extend_from_slice(&(-1.5_f64).to_le_bytes());
        expected.extend_from_slice(&(64.0_f64).to_le_bytes());
        expected.extend_from_slice(&(2.25_f64).to_le_bytes());
        expected.extend_from_slice(&(0.8_f32).to_le_bytes());
        expected.extend_from_slice(&(0.5_f32).to_le_bytes());
        expected.extend_from_slice(&(1.0_f64).to_le_bytes());
        expected.extend_from_slice(&(-2.0_f64).to_le_bytes());
        expected.extend_from_slice(&(0.75_f64).to_le_bytes());
        expected.extend_from_slice(&(-1.5_f64).to_le_bytes());
        assert_eq!(
            encoded, expected,
            "Rust encoding must match host schema-v1 bytes"
        );
        assert_eq!(
            from_bytes::<EntityKnockbackAttempt>(&encoded).unwrap(),
            event
        );
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
    fn cancellable_entity_breed_attempt_matches_host_schema_v1_bytes() {
        let event = EntityBreedAttempt {
            parent_a_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            parent_a_type_id: "minecraft:cow".to_owned(),
            parent_b_id: "00000000-0000-0000-0000-000000000002".to_owned(),
            parent_b_type_id: "minecraft:cow".to_owned(),
            child_id: "00000000-0000-0000-0000-000000000003".to_owned(),
            child_type_id: "minecraft:cow".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
        };
        let encoded = to_bytes(&event).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &event.parent_a_id);
        append_test_string(&mut expected, &event.parent_a_type_id);
        append_test_string(&mut expected, &event.parent_b_id);
        append_test_string(&mut expected, &event.parent_b_type_id);
        append_test_string(&mut expected, &event.child_id);
        append_test_string(&mut expected, &event.child_type_id);
        append_test_string(&mut expected, &event.dimension_id);
        assert_eq!(
            encoded, expected,
            "Rust encoding must match the host schema-v1 bytes"
        );
        assert_eq!(from_bytes::<EntityBreedAttempt>(&encoded).unwrap(), event);
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
    fn entity_tick_snapshot_round_trips_detached_identity_and_position() {
        let event = EntityTick {
            entity_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            entity_type_id: "minecraft:pig".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            x: -1.5,
            y: 64.0,
            z: 2.25,
            tick_count: 1234,
        };
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(from_bytes::<EntityTick>(&encoded).unwrap(), event);
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
    fn player_item_pickup_attempt_matches_host_schema_v1_and_round_trips_position() {
        let attempt = PlayerItemPickupAttempt {
            player_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            item_id: "minecraft:diamond".to_owned(),
            count: 3,
            x: -1.25,
            y: 64.0,
            z: 7.5,
        };
        let encoded = to_bytes(&attempt).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &attempt.player_id);
        append_test_string(&mut expected, &attempt.item_id);
        expected.extend_from_slice(&attempt.count.to_le_bytes());
        expected.extend_from_slice(&attempt.x.to_le_bytes());
        expected.extend_from_slice(&attempt.y.to_le_bytes());
        expected.extend_from_slice(&attempt.z.to_le_bytes());
        assert_eq!(
            encoded, expected,
            "Rust bytes must match host schema-v1 layout"
        );
        assert_eq!(
            from_bytes::<PlayerItemPickupAttempt>(&encoded).unwrap(),
            attempt
        );
    }

    #[test]
    fn player_fished_attempt_matches_host_schema_v1_and_snapshots_only_bounded_drop_ids() {
        let attempt = PlayerFishedAttempt {
            player_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            hook_x: 4.25,
            hook_y: 65.0,
            hook_z: -2.5,
            rod_damage: 3,
            drops: vec![
                FishedDropSnapshot {
                    item_id: "minecraft:cod".to_owned(),
                    count: 2,
                },
                FishedDropSnapshot {
                    item_id: "minecraft:bone".to_owned(),
                    count: 1,
                },
            ],
            drops_truncated: false,
        };
        let encoded = to_bytes(&attempt).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &attempt.player_id);
        append_test_string(&mut expected, &attempt.dimension_id);
        expected.extend_from_slice(&attempt.hook_x.to_le_bytes());
        expected.extend_from_slice(&attempt.hook_y.to_le_bytes());
        expected.extend_from_slice(&attempt.hook_z.to_le_bytes());
        expected.extend_from_slice(&attempt.rod_damage.to_le_bytes());
        expected.extend_from_slice(&2_u32.to_le_bytes());
        for drop in &attempt.drops {
            expected.extend_from_slice(&1_u32.to_le_bytes());
            append_test_string(&mut expected, &drop.item_id);
            expected.extend_from_slice(&drop.count.to_le_bytes());
        }
        expected.push(0);
        assert_eq!(
            encoded, expected,
            "Rust encoding must match host schema-v1 bytes"
        );
        assert_eq!(
            from_bytes::<PlayerFishedAttempt>(&encoded).unwrap(),
            attempt
        );
    }

    #[test]
    fn player_item_use_attempt_round_trips_detached_item_snapshot() {
        let attempt = PlayerItemUseAttempt {
            player_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            hand: "off_hand".to_owned(),
            item_id: "minecraft:ender_pearl".to_owned(),
            count: 3,
        };
        let encoded = to_bytes(&attempt).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &attempt.player_id);
        append_test_string(&mut expected, &attempt.dimension_id);
        append_test_string(&mut expected, &attempt.hand);
        append_test_string(&mut expected, &attempt.item_id);
        expected.extend_from_slice(&3_i32.to_le_bytes());
        assert_eq!(encoded, expected);
        assert_eq!(
            from_bytes::<PlayerItemUseAttempt>(&encoded).unwrap(),
            attempt
        );

        let empty_hand = PlayerItemUseAttempt {
            hand: "main_hand".to_owned(),
            item_id: "minecraft:air".to_owned(),
            count: 0,
            ..attempt
        };
        let encoded = to_bytes(&empty_hand).unwrap();
        assert_eq!(
            from_bytes::<PlayerItemUseAttempt>(&encoded).unwrap(),
            empty_hand
        );
    }

    #[test]
    fn potion_use_attempt_matches_host_schema_and_bounds_values() {
        let attempt = PlayerPotionUseAttempt {
            player_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            item_id: "minecraft:potion".to_owned(),
            count: 1,
            duration_ticks: 32,
            hand: "off_hand".to_owned(),
        };
        let encoded = to_bytes(&attempt).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &attempt.player_id);
        append_test_string(&mut expected, &attempt.dimension_id);
        append_test_string(&mut expected, &attempt.item_id);
        expected.extend_from_slice(&1_u32.to_le_bytes());
        expected.extend_from_slice(&32_u32.to_le_bytes());
        append_test_string(&mut expected, &attempt.hand);
        assert_eq!(
            encoded, expected,
            "Rust bytes must match the host schema-v1 payload"
        );
        assert_eq!(
            from_bytes::<PlayerPotionUseAttempt>(&encoded).unwrap(),
            attempt
        );

        let maximums = PlayerPotionUseAttempt {
            player_id: attempt.player_id.clone(),
            dimension_id: attempt.dimension_id.clone(),
            item_id: format!("test:{}", "a".repeat(251)),
            count: MAX_PLAYER_POTION_USE_ITEM_COUNT,
            duration_ticks: MAX_PLAYER_POTION_USE_DURATION_TICKS,
            hand: "main_hand".to_owned(),
        };
        let encoded_maximums = to_bytes(&maximums).unwrap();
        assert_eq!(
            from_bytes::<PlayerPotionUseAttempt>(&encoded_maximums).unwrap(),
            maximums
        );

        for action in [
            PlayerPotionUseAttemptAction::Pass,
            PlayerPotionUseAttemptAction::Cancel,
        ] {
            let encoded = to_bytes(&action).unwrap();
            assert_eq!(encoded.len(), 1);
            assert_eq!(
                from_bytes::<PlayerPotionUseAttemptAction>(&encoded).unwrap(),
                action
            );
        }
        assert!(from_bytes::<PlayerPotionUseAttemptAction>(&[2]).is_err());

        let mut invalid_count = attempt.clone();
        invalid_count.count = MAX_PLAYER_POTION_USE_ITEM_COUNT + 1;
        assert_eq!(
            to_bytes(&invalid_count).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        let mut invalid_duration = attempt.clone();
        invalid_duration.duration_ticks = MAX_PLAYER_POTION_USE_DURATION_TICKS + 1;
        assert_eq!(
            to_bytes(&invalid_duration).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        let mut invalid_hand = attempt;
        invalid_hand.hand = "head".to_owned();
        assert_eq!(
            to_bytes(&invalid_hand).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        let mut invalid_dimension = maximums.clone();
        invalid_dimension.dimension_id = "overworld".to_owned();
        assert_eq!(
            to_bytes(&invalid_dimension).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        let mut invalid_item_id = maximums.clone();
        invalid_item_id.item_id = "potion".to_owned();
        assert_eq!(
            to_bytes(&invalid_item_id).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        let oversized_item_id = PlayerPotionUseAttempt {
            item_id: format!("test:{}", "a".repeat(252)),
            ..maximums
        };
        assert_eq!(
            to_bytes(&oversized_item_id).unwrap_err().code(),
            ErrorCode::PayloadTooLarge
        );
    }

    #[test]
    fn player_totem_use_attempt_matches_host_schema_and_validates_detached_source() {
        let attempt = PlayerTotemUseAttempt {
            player_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            damage_type_id: "minecraft:player_attack".to_owned(),
            source_entity_id: Some("00000000-0000-0000-0000-000000000002".to_owned()),
            source_entity_type_id: Some("minecraft:zombie".to_owned()),
            totem_id: "minecraft:totem_of_undying".to_owned(),
            count: 1,
            hand: "off_hand".to_owned(),
        };
        let encoded = to_bytes(&attempt).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &attempt.player_id);
        append_test_string(&mut expected, &attempt.dimension_id);
        append_test_string(&mut expected, &attempt.damage_type_id);
        append_test_string(&mut expected, attempt.source_entity_id.as_ref().unwrap());
        append_test_string(
            &mut expected,
            attempt.source_entity_type_id.as_ref().unwrap(),
        );
        append_test_string(&mut expected, &attempt.totem_id);
        expected.extend_from_slice(&attempt.count.to_le_bytes());
        append_test_string(&mut expected, &attempt.hand);
        assert_eq!(
            encoded, expected,
            "Rust bytes must match host schema-v1 layout"
        );
        assert_eq!(
            from_bytes::<PlayerTotemUseAttempt>(&encoded).unwrap(),
            attempt
        );

        let environmental = PlayerTotemUseAttempt {
            damage_type_id: "minecraft:lava".to_owned(),
            source_entity_id: None,
            source_entity_type_id: None,
            hand: "main_hand".to_owned(),
            ..attempt.clone()
        };
        let encoded = to_bytes(&environmental).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &environmental.player_id);
        append_test_string(&mut expected, &environmental.dimension_id);
        append_test_string(&mut expected, &environmental.damage_type_id);
        expected.extend_from_slice(&u32::MAX.to_le_bytes());
        expected.extend_from_slice(&u32::MAX.to_le_bytes());
        append_test_string(&mut expected, &environmental.totem_id);
        expected.extend_from_slice(&environmental.count.to_le_bytes());
        append_test_string(&mut expected, &environmental.hand);
        assert_eq!(
            encoded, expected,
            "None must use the host's -1 length sentinel"
        );
        assert_eq!(
            from_bytes::<PlayerTotemUseAttempt>(&encoded).unwrap(),
            environmental
        );

        for action in [
            PlayerTotemUseAttemptAction::Pass,
            PlayerTotemUseAttemptAction::Cancel,
        ] {
            let encoded = to_bytes(&action).unwrap();
            assert_eq!(encoded.len(), 1);
            assert_eq!(
                from_bytes::<PlayerTotemUseAttemptAction>(&encoded).unwrap(),
                action
            );
        }
        assert!(from_bytes::<PlayerTotemUseAttemptAction>(&[2]).is_err());

        let mut invalid_pair = environmental.clone();
        invalid_pair.source_entity_id = Some("00000000-0000-0000-0000-000000000002".to_owned());
        assert_eq!(
            to_bytes(&invalid_pair).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        let mut invalid_count = environmental.clone();
        invalid_count.count = MAX_PLAYER_TOTEM_USE_COUNT + 1;
        assert_eq!(
            to_bytes(&invalid_count).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        let mut invalid_type = environmental;
        invalid_type.damage_type_id = "lava".to_owned();
        assert_eq!(
            to_bytes(&invalid_type).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn module_item_use_round_trips_owned_item_action() {
        let attempt = ModuleItemUse {
            player_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            hand: "main_hand".to_owned(),
            item_id: "sjml_game_test:content/gametest_ruby".to_owned(),
            count: 3,
        };
        let encoded = to_bytes(&attempt).unwrap();
        assert_eq!(from_bytes::<ModuleItemUse>(&encoded).unwrap(), attempt);
        for action in [
            ItemUseAction::Pass,
            ItemUseAction::Success,
            ItemUseAction::ConsumeOne,
        ] {
            assert_eq!(
                from_bytes::<ItemUseAction>(&to_bytes(&action).unwrap()).unwrap(),
                action
            );
        }
        assert!(from_bytes::<ItemUseAction>(&[3]).is_err());
    }

    #[test]
    fn module_item_use_on_block_round_trips_held_item_and_target() {
        let use_event = ModuleItemUseOnBlock {
            player_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            item_id: "sjml_game_test:content/gametest_ruby".to_owned(),
            item_count: 2,
            block_id: "minecraft:stone".to_owned(),
            x: -2,
            y: 64,
            z: 17,
            face: "north".to_owned(),
            hand: "main_hand".to_owned(),
        };
        let encoded = to_bytes(&use_event).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &use_event.player_id);
        append_test_string(&mut expected, &use_event.dimension_id);
        append_test_string(&mut expected, &use_event.item_id);
        expected.extend_from_slice(&2_i32.to_le_bytes());
        append_test_string(&mut expected, &use_event.block_id);
        expected.extend_from_slice(&(-2_i32).to_le_bytes());
        expected.extend_from_slice(&64_i32.to_le_bytes());
        expected.extend_from_slice(&17_i32.to_le_bytes());
        append_test_string(&mut expected, &use_event.face);
        append_test_string(&mut expected, &use_event.hand);
        assert_eq!(encoded, expected);
        assert_eq!(
            from_bytes::<ModuleItemUseOnBlock>(&encoded).unwrap(),
            use_event
        );
        assert_eq!(
            from_bytes::<ItemUseAction>(&to_bytes(&ItemUseAction::ConsumeOne).unwrap()).unwrap(),
            ItemUseAction::ConsumeOne
        );
    }

    #[test]
    fn module_item_use_on_entity_round_trips_held_item_and_target() {
        let use_event = ModuleItemUseOnEntity {
            player_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            item_id: "sjml_game_test:content/gametest_ruby".to_owned(),
            item_count: 2,
            target_entity_id: "00000000-0000-0000-0000-000000000002".to_owned(),
            target_type_id: "minecraft:cow".to_owned(),
            target_x: -3.5,
            target_y: 64.0,
            target_z: 17.25,
            hand: "main_hand".to_owned(),
        };
        let encoded = to_bytes(&use_event).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &use_event.player_id);
        append_test_string(&mut expected, &use_event.dimension_id);
        append_test_string(&mut expected, &use_event.item_id);
        expected.extend_from_slice(&2_i32.to_le_bytes());
        append_test_string(&mut expected, &use_event.target_entity_id);
        append_test_string(&mut expected, &use_event.target_type_id);
        expected.extend_from_slice(&(-3.5_f64).to_le_bytes());
        expected.extend_from_slice(&64.0_f64.to_le_bytes());
        expected.extend_from_slice(&17.25_f64.to_le_bytes());
        append_test_string(&mut expected, &use_event.hand);
        assert_eq!(encoded, expected);
        assert_eq!(
            from_bytes::<ModuleItemUseOnEntity>(&encoded).unwrap(),
            use_event
        );
        assert_eq!(
            from_bytes::<ItemUseAction>(&to_bytes(&ItemUseAction::ConsumeOne).unwrap()).unwrap(),
            ItemUseAction::ConsumeOne
        );
    }

    #[test]
    fn module_block_use_round_trips_detached_target_and_action() {
        let use_event = ModuleBlockUse {
            player_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            block_id: "sjml_game_test:content/gametest_interactive_block".to_owned(),
            x: -12,
            y: 68,
            z: 301,
            face: "up".to_owned(),
            hand: "main_hand".to_owned(),
            item_id: "minecraft:diamond".to_owned(),
            item_count: 3,
        };
        let encoded = to_bytes(&use_event).unwrap();
        let mut expected = 1_i32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &use_event.player_id);
        append_test_string(&mut expected, &use_event.dimension_id);
        append_test_string(&mut expected, &use_event.block_id);
        expected.extend_from_slice(&(-12_i32).to_le_bytes());
        expected.extend_from_slice(&68_i32.to_le_bytes());
        expected.extend_from_slice(&301_i32.to_le_bytes());
        append_test_string(&mut expected, &use_event.face);
        append_test_string(&mut expected, &use_event.hand);
        append_test_string(&mut expected, &use_event.item_id);
        expected.extend_from_slice(&3_i32.to_le_bytes());
        assert_eq!(encoded, expected);
        assert_eq!(from_bytes::<ModuleBlockUse>(&encoded).unwrap(), use_event);

        for action in [
            BlockUseAction::Pass,
            BlockUseAction::Success,
            BlockUseAction::ConsumeOne,
        ] {
            assert_eq!(
                from_bytes::<BlockUseAction>(&to_bytes(&action).unwrap()).unwrap(),
                action
            );
        }
        assert!(from_bytes::<BlockUseAction>(&[3]).is_err());
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
    fn player_tick_snapshot_round_trips_its_player_dimension_and_tick_count() {
        let event = PlayerTick {
            player_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            dimension_id: "minecraft:the_nether".to_owned(),
            tick_count: 1234,
        };
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(from_bytes::<PlayerTick>(&encoded).unwrap(), event);
    }

    #[test]
    fn client_tick_snapshot_round_trips_its_tick_count() {
        let event = ClientTick { tick_count: 1234 };
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(from_bytes::<ClientTick>(&encoded).unwrap(), event);
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
    fn player_item_crafted_snapshot_round_trips_recipe_and_bounded_result_data() {
        let event = PlayerItemCrafted {
            player_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            recipe_id: "minecraft:oak_planks".to_owned(),
            item_id: "minecraft:oak_planks".to_owned(),
            count: 4,
        };
        let encoded = to_bytes(&event).unwrap();
        assert_eq!(from_bytes::<PlayerItemCrafted>(&encoded).unwrap(), event);
    }

    #[test]
    fn player_villager_trade_snapshot_round_trips_bounded_offer_and_merchant_data() {
        let event = PlayerVillagerTraded {
            player_id: "00000000-0000-0000-0000-000000000001".to_owned(),
            merchant_id: "00000000-0000-0000-0000-000000000002".to_owned(),
            merchant_type_id: "minecraft:villager".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            x: 3,
            y: 64,
            z: -5,
            profession_id: "minecraft:librarian".to_owned(),
            profession_level: 2,
            cost_a_item_id: "minecraft:emerald".to_owned(),
            cost_a_count: 4,
            cost_b_item_id: String::new(),
            cost_b_count: 0,
            result_item_id: "minecraft:book".to_owned(),
            result_count: 1,
            uses: 1,
            max_uses: 12,
        };
        let encoded = to_bytes(&event).unwrap();
        let mut expected = 1_u32.to_le_bytes().to_vec();
        append_test_string(&mut expected, &event.player_id);
        append_test_string(&mut expected, &event.merchant_id);
        append_test_string(&mut expected, &event.merchant_type_id);
        append_test_string(&mut expected, &event.dimension_id);
        expected.extend_from_slice(&event.x.to_le_bytes());
        expected.extend_from_slice(&event.y.to_le_bytes());
        expected.extend_from_slice(&event.z.to_le_bytes());
        append_test_string(&mut expected, &event.profession_id);
        expected.extend_from_slice(&event.profession_level.to_le_bytes());
        append_test_string(&mut expected, &event.cost_a_item_id);
        expected.extend_from_slice(&event.cost_a_count.to_le_bytes());
        append_test_string(&mut expected, &event.cost_b_item_id);
        expected.extend_from_slice(&event.cost_b_count.to_le_bytes());
        append_test_string(&mut expected, &event.result_item_id);
        expected.extend_from_slice(&event.result_count.to_le_bytes());
        expected.extend_from_slice(&event.uses.to_le_bytes());
        expected.extend_from_slice(&event.max_uses.to_le_bytes());
        assert_eq!(
            encoded, expected,
            "Rust encoding must match the host schema-v1 bytes"
        );
        assert_eq!(from_bytes::<PlayerVillagerTraded>(&encoded).unwrap(), event);
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
        assert_eq!(
            subscribe("player.tick", EventPriority::Normal)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            subscribe("client.tick", EventPriority::Normal)
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
        assert_eq!(
            subscribe_with_interval("player.tick", EventPriority::Normal, 20)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            subscribe_with_interval("player.tick", EventPriority::Normal, 1)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            subscribe_with_interval("player.tick", EventPriority::Normal, 1_200)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            subscribe_with_interval("entity.tick", EventPriority::Normal, 1)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            subscribe_with_interval("entity.tick", EventPriority::Normal, 1_200)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            subscribe_with_interval("entity.tick", EventPriority::Normal, 0)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            subscribe_with_interval("player.tick", EventPriority::Normal, 0)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            subscribe_with_interval(
                "player.tick",
                EventPriority::Normal,
                MAX_SERVER_TICK_INTERVAL + 1
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            subscribe_with_interval("client.tick", EventPriority::Normal, 0)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            subscribe_with_interval(
                "client.tick",
                EventPriority::Normal,
                MAX_SERVER_TICK_INTERVAL + 1
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            subscribe_with_interval("client.tick", EventPriority::Normal, 20)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            subscribe_with_interval("client.tick", EventPriority::Normal, 1)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            subscribe_with_interval("client.tick", EventPriority::Normal, 1_200)
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
